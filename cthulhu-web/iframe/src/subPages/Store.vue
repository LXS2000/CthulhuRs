<template>
  <el-scrollbar height="100%" style="height: 100%">
    <div class="row" style="margin-top: 10px">
      <!--   trees-->
      <div class="names col">
        <div v-for="name in names" class="row name" @click="nameClick(name)"
             :style="`color:${name===active?'#5cf1f6':'#c7c7c7'}`">
          <el-icon style="width: 20%;margin: auto">
            <Coin/>
          </el-icon>
          <span class="omit" style="width: 80%">{{ name }}</span>
        </div>
      </div>
      <div class="col" style="width: 76%;">
        <div class="row" style="margin-bottom: 0.3rem">
          <span style="font-size: 0.8rem;color:#c5c5c5;font-weight: bold;">TREE：</span>
          <span style="font-weight: bold;font-size: 0.8rem;color: #b2b2b2">{{ active }}</span>
        </div>
        <div class="row" style="background: #262626;font-size: 0.9rem;font-weight: bold;color: #c7c7c7">
          <span style="width: 30%">{{ tl('k') }}</span>
          <span style="width: 70%">{{ tl('v') }}</span>
        </div>
        <div class="kvs col">
          <el-scrollbar>
            <div v-for="kv in nameMapKvs[active]" class="row" @click="kvClick(kv[0],kv[1])"
                 :style="`background:${kv[0]===theKey?'rgba(121,120,120,0.92)':''};color:${kv[0]===theKey?'rgb(245,245,245)':'#b6b6b6'}`">
              <span class="key omit">{{ kv[0] }}</span>
              <span class="value omit">{{ kv[1] }}</span>
            </div>
          </el-scrollbar>
        </div>
        <div class="row" style="margin-bottom: 0.3rem ;margin-top: 1rem">
          <span style="font-size: 0.9rem;color:#c5c5c5;font-weight: bold;">KEY：</span>
          <span style="font-weight: bold;font-size: 0.9rem;color: #b2b2b2">{{ theKey }}</span>
        </div>
        <div class="valueContent col">
          <el-scrollbar style="padding: 0.3rem">
            {{ theValue }}
          </el-scrollbar>
        </div>
        <div style="height: 1rem"></div>
      </div>
    </div>
    <div style="height: 3rem"/>
  </el-scrollbar>

</template>
<script setup>
import {useStore} from "vuex";
import {treeList, treeNames} from "/@/api/api.js";
import {reactive, ref} from "vue";
import {tl,} from "/@/local.js";


const store = useStore();
const curPlugin = store.state.curPlugin;

const names = reactive([])
const active = ref("")
const nameMapKvs = reactive({})
const theKey = ref("")
const theValue = ref("")

async function nameClick(name) {
  active.value = name
  let kvs = nameMapKvs[name];
  if (!kvs) {
    await treeList(curPlugin.id, name).then(list => {
      nameMapKvs[name] = list;
      kvs = list;
    })
  }
  let kv = kvs[0] || []
  kvClick(kv[0], kv[1])
}

function kvClick(k, v) {
  theKey.value = k;
  theValue.value = v;
}

treeNames(curPlugin.id).then(async list => {
  names.splice(0, names.length)
  names.push(...list);
  active.value = names[0]
  await nameClick(active.value)
  let kv = nameMapKvs[active.value][0] || [];
  kvClick(kv[0], kv[1])
})
</script>

<style scoped lang="less">
.names {
  width: 20%;
  height: 100%;
}

.name {
  width: 90%;
  margin: 0.3rem auto;
  font-weight: bold;
  font-size: 1rem;
}

.key {
  border-right: 2px solid white;
  border-bottom: 1px solid white;
  width: 30%;

}

.value {
  border-right: 2px solid white;
  border-bottom: 1px solid white;
  width: 70%;
}

.kvs {
  border: 1px white solid;
  border-radius: 1px;
  height: 12.4rem;
}

.valueContent {
  margin-top: auto;
  height: 15rem;
  color: #c7c7c7;
  border: 1px solid #9b9b9b;
  border-radius: 5px;
}

</style>
