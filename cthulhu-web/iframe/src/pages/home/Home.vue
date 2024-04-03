<template>
  <el-scrollbar height="100%" id="home-wrap" class="col" v-loading="loading"
                element-loading-background="rgba(122, 122, 122, 0.3)">
    <div class="col-between card">
      <div style="color: white;"> {{ info.email }}</div>

      <div class="row">
        <SvgIcon name="yes" :color="'#87e861'" size="1.2"/>
        <span style="color: white;margin-left: 0.1rem">{{ tl("running") }}</span>
        <div class="row" style="align-items: center;margin-left: auto">
          <TextBtn :name="tl('reboot')" size="0.7rem" :color="'#f87373'" style="margin-left: 0.5rem" @click="onRestart"/>
        </div>
      </div>
    </div>
    <div class="col card">
      <div style="color: white;margin-left: 0.1rem">{{ tl("pAddr") }}：</div>
      <div style="margin-left: 1rem">
        <div class="row" style="align-items: center;justify-content: space-between">
          <span style="color: white;"> http：</span>
          <div class="row" style="align-items: center">
            <span style="font-size: 0.6rem;color: white;margin-left: 0.5rem">{{ http }}</span>
            <IconBtn :size="1" name="copy" :color="'#66cff5'" style="margin-left: 0.2rem" @click="clip(http)"/>
          </div>
        </div>
        <div class="row" style="align-items: center;justify-content: space-between">
          <span style="color: white;"> tcp：</span>
          <div class="row" style="align-items: center">
            <span style="font-size: 0.6rem;color: white;margin-left: 0.5rem">{{ tcp }}</span>
            <IconBtn :size="1" name="copy" :color="'#66cff5'" style="margin-left: 0.2rem" @click="clip(tcp)"/>
          </div>
        </div>
      </div>
    </div>
    <div id="clients" class="col card">
      <div style="color: white;font-weight: bold;margin-bottom: 0.5rem">{{ tl("clients") }}:</div>
      <el-table :data="info.scopes" style="height: 96%;width: 100%;margin-bottom: auto"
                :header-cell-style="{backgroundColor:'#181818',color:'#6b6b6b'}">
        <el-table-column fixed="left" prop="id" label="id" width="180"/>
        <el-table-column prop="ip" label="ip" width="100"/>
        <el-table-column prop="scheme" label="scheme" width="100"/>
        <el-table-column prop="host" label="host" width="180"/>
        <el-table-column prop="ua" label="ua" width="300"/>
        <el-table-column prop="email" label="email" width="100">
          <template #default="scope">
            {{ scope.row.email || '-' }}
          </template>
        </el-table-column>
        <el-table-column prop="tab" label="tab" width="100">
          <template #default="scope">
            {{ scope.row.tab || '-' }}
          </template>
        </el-table-column>
        <el-table-column prop="frame" label="frame" width="100">
          <template #default="scope">
            {{ scope.row.frame || '-' }}
          </template>
        </el-table-column>
        <el-table-column prop="custom" label="custom" width="180">
          <template #default="scope">
            {{ scope.row.custom || '-' }}
          </template>
        </el-table-column>
      </el-table>
    </div>
  </el-scrollbar>
</template>

<script setup>
import {inject, onMounted, ref, computed, reactive} from "vue";
import SvgIcon from "../../components/SvgIcon.vue";
import IconBtn from "../../components/IconBtn.vue";
import {serverInfo, restart} from "../../api/api.js";
import TextBtn from "../../components/TextBtn.vue";
import {ElLoading, ElMessage} from "element-plus";

import Local from "/@/local.js";

const local = new Local({
  reboot: {zh: "重启", en: "reboot"},
  rebooting: {zh: "重启中", en: "rebooting"},
  running: {zh: "运行中", en: "running"},
  pAddr: {zh: "代理地址", en: "proxy address"},
  clients: {zh: "连接中的客户端", en: "connecting clients"},
  notSpt: {zh: "浏览器粘贴板不支持", en: "browser clipboard not supported" },
  copyTo :{zh: "内容已复制到粘贴板", en: "content copied to the clipboard" },
  cf :{zh: "复制失败", en: "copy failed" },
},true);
const tlp = local.tlp.bind(local);
const tlf = local.tlf.bind(local);
const tl = local.tl.bind(local);

const winSize = inject('winSize');
const info = reactive({
  addr: "127.0.0.1",
  port: 3000,
  scopes: [],
  email: "",
  integrals: 0
});
const clients = ref([])

const loading = ref(false)


let http = computed(() => {
  let {addr, port} = info
  return `http://${addr}:${port}`
})
let tcp = computed(() => {
  let {addr, port} = info
  return `${addr}:${port}`
})


onMounted(async () => {
  loading.value = true;

  await serverInfo().then(res => {
    Object.keys((res || {})).forEach(k => info[k] = res[k])
  })
  loading.value = false;
})

async function onRestart() {
  let load = ElLoading.service({fullscreen: true, text: tlp("{rebooting}...")})
  try {
    await restart()
  } catch (e) {
  }
  load.close()
}

function clip(text) {
  if (!navigator.clipboard) {
    ElMessage.error(tl("notSpt"))
    return
  }
  navigator.clipboard.writeText(text).then(res => {
    ElMessage.success(tl("copyTo"))
  }).catch(e => {
    console.log(e)
    ElMessage.error(tl("cf"))
  })
}

</script>

<style scoped lang="less">
.card {
  width: 90%;
  margin: 0.8rem auto;
  //background-color: #2f2f2f;
  border-radius: 0.5rem;
  padding: 0.2rem 0.3rem;
  box-shadow: 1px 1px 2px 1px gray;
}

#home-wrap {
  width: 98%;
  padding: 0.2rem 0.6rem;
}

#clients {
  height: 25rem;
}

.wrap {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: flex-start;
}

.item {
  width: 7.2rem;
  border: 0.3rem solid transparent;
  flex-shrink: 0;
}


.pkgs {
  margin: auto;
  flex-wrap: wrap;
  justify-content: space-between;
}

.pkg {
  width: 11vw;
  height: 11vw;
  min-width: 100px;
  min-height: 100px;
  border-radius: 2vw;
  background-image: url("../../assets/pkg_bg.png");
  background-size: cover;
  align-items: center;
  margin: 0 auto 0.6rem;
}

.key {
  font-size: 1.2rem;
  color: gray;
}

.val {
  font-size: 1rem;
}


/deep/ .el-table__header-wrapper {
  height: 2rem;
}

/deep/ .el-table__body .el-table__row.hover-row td {
  color: white;
  background-color: rgba(91, 91, 91, 0.93);
}

/deep/ .el-table {
  background-color: transparent;
}

/deep/ .el-table__row {
  color: #c7c7c7;
  background-color: #202020;
}

/deep/ .el-table__header-wrapper {
  background-color: transparent;
}

/deep/ .el-overlay-dialog {
  overflow: hidden;
}

.modal {
  width: 98%;
  height: 98%;
  margin: auto;
}
</style>
