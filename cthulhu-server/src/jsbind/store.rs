use std::collections::HashMap;

use rquickjs::{class::Trace, CatchResultExt, Class, Ctx, Result as JsResult};
use sled::{
    transaction::{self, abort, ConflictableTransactionError},
    Batch, Db, Error, Tree,
};

use crate::{auto_option, auto_result, jsbind};

use super::to_js_err;

fn to_value<'js>(ctx: Ctx<'js>, json: sled::IVec) -> JsResult<rquickjs::Value<'js>> {
    let value = if json.is_empty() {
        rquickjs::Value::new_undefined(ctx.clone())
    } else {
        let json =
            String::from_utf8(json.to_vec()).map_err(|e| jsbind::to_js_err(e, ctx.clone()))?;
        ctx.json_parse(json)?
    };
    Ok(value)
}
#[rquickjs::class(rename = "Tree")]
#[derive(Debug, Trace)]
pub struct JsTree {
    pub name: String,
    #[qjs(skip_trace)]
    pub tree: Tree,
}
#[rquickjs::methods]
impl JsTree {
    #[qjs(constructor)]
    pub fn new(ctx: rquickjs::Ctx<'_>) -> rquickjs::Result<Self> {
        Err(jsbind::throw_js_err("Illegal constructor",ctx))
    }
    #[qjs(rename = "set")]
    pub fn set<'js>(
        &self,
        key: String,
        value: rquickjs::Value<'js>,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<()> {
        let json = auto_option!(ctx.json_stringify(value)?, Ok(()));
        let json = json.to_string()?;
        self.tree
            .insert(key, json.as_bytes())
            .map_err(|e| jsbind::to_js_err(e, ctx.clone()))?;
        Ok(())
    }
    #[qjs(rename = "sets")]
    pub fn sets<'js>(
        &self,
        kvs: HashMap<String, rquickjs::Value<'js>>,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<()> {
        let res=self.tree.transaction(|tree|{
            let mut batch = Batch::default();
            for (key, value) in &kvs{
                let json=auto_result!(ctx.json_stringify(value)
                .map_err(|e|jsbind::to_js_err(e,ctx.clone())),err=>{return abort(err)});
                let json=auto_option!(json,{
                    continue
                }) ;
                let json=json.to_string().map_err(|e|jsbind::to_js_err(e,ctx.clone()));
                let json=auto_result!(json,err=>{return abort(err)});
                batch.insert(&key as &str, json.as_bytes());
            }
            auto_result!( tree.apply_batch(&batch).map_err(|e|jsbind::to_js_err(e,ctx.clone())),err=>{return abort(err)});
            Ok(())
        });
        Ok(res.map_err(|e| jsbind::to_js_err(e, ctx.clone()))?)
    }
    #[qjs(rename = "get")]
    pub fn get<'js>(&self, key: String, ctx: Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let json: sled::IVec = self
            .tree
            .get(key)
            .map_err(|e| jsbind::to_js_err(e, ctx.clone()))?
            .unwrap_or_default();
        to_value(ctx, json)
    }
    #[qjs(rename = "gets")]
    pub fn gets<'js>(
        &self,
        keys: Vec<String>,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<HashMap<String, rquickjs::Value<'js>>> {
        let mut map = HashMap::new();
        for key in keys {
            let val = self.get(key.clone(), ctx.clone())?;
            map.insert(key, val);
        }
        Ok(map)
    }
    #[qjs(rename = "getWith")]
    pub fn get_with<'js>(
        &self,
        prefix: String,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<HashMap<String, rquickjs::Value<'js>>> {
        let mut map = HashMap::new();
        for item in self.tree.scan_prefix(&prefix) {
            let (k, v) = item.map_err(|e| jsbind::to_js_err(e, ctx.clone()))?;
            let key = String::from_utf8(k.to_vec()).unwrap_or_default();
            let value = to_value(ctx.clone(), v)?;
            map.insert(key, value);
        }
        Ok(map)
    }
    #[qjs(rename = "remove")]
    pub fn remove<'js>(
        &self,
        key: String,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<Option<rquickjs::Value<'js>>> {
        let vec = self
            .tree
            .remove(key)
            .map_err(|e| jsbind::to_js_err(e, ctx.clone()))?;
        let vec = auto_option!(vec, Ok(None));
        let value = to_value(ctx.clone(), vec)?;
        Ok(Some(value))
    }
    #[qjs(rename = "removes")]
    pub fn removes<'js>(
        &self,
        keys: Vec<String>,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<HashMap<String, rquickjs::Value<'js>>> {
        let res=self.tree.transaction(|tree|{
            let mut batch = Batch::default();
            let mut map = HashMap::new();
            for key in &keys {
                let value =auto_result!(self.get(key.to_string(),ctx.clone()),err=>{return abort(err)});
                batch.remove(key.as_str());
                map.insert(key.to_string(), value);
            }
            auto_result!( tree.apply_batch(&batch).map_err(|e|jsbind::to_js_err(e,ctx.clone())),err=>{return abort(err)});
            Ok(map)
        });
        let map = res.map_err(|e| jsbind::to_js_err(e, ctx.clone()))?;
        Ok(map)
    }
    #[qjs(rename = "removeWith")]
    pub fn removes_with<'js>(
        &self,
        prefix: String,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<HashMap<String, rquickjs::Value<'js>>> {
        let res=self.tree.transaction(|tree|{
            let mut batch = Batch::default();
            let mut map = HashMap::new();
            for item in self.tree.scan_prefix(&prefix) {
                let (k, v) = auto_result!(item.map_err(|e| jsbind::to_js_err(e, ctx.clone())),err=>{return abort(err)}) ;
                let key = String::from_utf8(k.to_vec()).unwrap_or_default();
                batch.remove(key.as_str());
                let value=auto_result!(to_value(ctx.clone(), v),err=>{return abort(err)}) ;
                map.insert(key, value);
            }
            let res=tree.apply_batch(&batch).map_err(|e| jsbind::to_js_err(e, ctx.clone()));
            auto_result!( res,err=>{return abort(err)});
            Ok(map)
        });
        Ok(res.map_err(|e| jsbind::to_js_err(e, ctx.clone()))?)
    }

    #[qjs(rename = "iterWith")]
    pub fn iter_with<'js>(
        &self,
        prefix: String,
        func: rquickjs::Function<'js>,
        ctx: Ctx<'js>,
    ) -> JsResult<()> {
        for item in self.tree.scan_prefix(prefix) {
            let (k, v) = item.map_err(|e| jsbind::to_js_err(e, ctx.clone()))?;
            let key = String::from_utf8(k.to_vec()).unwrap_or_default();
            let value = to_value(ctx.clone(), v)?;
            let op = auto_result!( func.call::<_, Option<bool>>((key,value)).catch(&ctx),err=>{
                return Err( jsbind::handle_js_error(err,&ctx));
            })
            .unwrap_or(true);
            if !op {
                break;
            }
        }
        Ok(())
    }
    #[qjs(rename = "countWith")]
    pub fn count_with(&self, prefix: String) -> usize {
        self.tree.scan_prefix(prefix).count()
    }
    #[qjs(rename = "clear")]
    pub fn clear(&self, ctx: Ctx<'_>) -> rquickjs::Result<()> {
        self.tree
            .clear()
            .map_err(|e| jsbind::to_js_err(e, ctx.clone()))
    }
    #[qjs(rename = "flush")]
    pub fn flush(&self, ctx: Ctx<'_>) -> rquickjs::Result<()> {
        self.tree.flush().map_err(|e| jsbind::to_js_err(e, ctx))?;
        Ok(())
    }
    #[qjs(rename = "keys")]
    pub fn keys(&self, ctx: Ctx<'_>) -> rquickjs::Result<Vec<String>> {
        let mut keys: Vec<String> = vec![];
        for item in self.tree.iter() {
            let (k, _) = item.map_err(|e| jsbind::to_js_err(e, ctx.clone()))?;
            let key = String::from_utf8(k.to_vec()).unwrap_or_default();
            keys.push(key);
        }
        Ok(keys)
    }
    #[qjs(rename = "contains")]
    pub fn contains(&self, key: String, ctx: Ctx<'_>) -> JsResult<bool> {
        self.tree
            .contains_key(key)
            .map_err(|e| jsbind::to_js_err(e, ctx))
    }
    #[qjs(rename = "transaction")]
    pub fn transaction<'js>(&self, func: rquickjs::Function<'js>, ctx: Ctx<'js>) -> JsResult<rquickjs::Value<'js>> {
        let res: Result<rquickjs::Value<'js>, transaction::TransactionError<_>> = self.tree.transaction(|_db| {
            let val = auto_result!(func.call::<_, rquickjs::Value<'js>>(()).catch(&ctx),err=>{
                 let err=jsbind::handle_js_error(err,&ctx);
                 return Err(ConflictableTransactionError::Abort(err));
            });
            Ok(val)
        });
        match res {
            Ok(v) => Ok(v),
            Err(e) => match e {
                transaction::TransactionError::Abort(v) => Err(v),
                transaction::TransactionError::Storage(e) => Err(to_js_err(e, ctx.clone())),
            },
        }
    }
}

#[rquickjs::class(rename = "Store")]
#[derive(Debug, Trace)]
pub struct JsStore {
    pub id: String,
    #[qjs(skip_trace)]
    pub db: Db,
}
#[rquickjs::methods]
impl JsStore {
    #[qjs(rename = "openTree")]
    pub fn open_tree(&self, name: String, ctx: Ctx<'_>) -> rquickjs::Result<JsTree> {
        let tree = self
            .db
            .open_tree(&name)
            .map_err(|e: Error| jsbind::to_js_err(e, ctx))?;
        Ok(JsTree { name, tree })
    }
    #[qjs(rename = "dropTree")]
    pub fn drop_tree(&self, name: String, ctx: Ctx<'_>) -> rquickjs::Result<bool> {
        let b = self
            .db
            .drop_tree(&name)
            .map_err(|e: Error| jsbind::to_js_err(e, ctx))?;
        Ok(b)
    }
    #[qjs(rename = "treeNames")]
    pub fn tree_names(&self) -> Vec<String> {
        self.db
            .tree_names()
            .into_iter()
            .map(|v| String::from_utf8(v.to_vec()).unwrap())
            .collect::<Vec<String>>()
    }
    #[qjs(rename = "toString")]
    pub fn to_string(&self) -> String {
       format!("{:?}",&self)
    }
}
pub fn init_def(id: &str, ctx: &Ctx<'_>, db: Db) -> rquickjs::Result<()> {
    let store = JsStore {
        id: id.to_string(),
        db,
    };
    let cls = Class::instance(ctx.clone(), store)?;
    ctx.globals().set("store", cls)?;
    Ok(())
}
