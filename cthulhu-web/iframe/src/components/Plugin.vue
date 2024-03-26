<template>
  <div class="plugin col" :style="style">

    <div class="col" style="height: 100%;width: 100%">
      <!--        第一行-->
      <div class="row head">
        <img :src="pLogo" alt="logo" class="logo">
        <span class="name">{{ plugin.name }}</span>
        <span class="version">Version：{{ plugin.version }}</span>
      </div>
      <!--      第二行-->
      <div class="row" style="height: 80%">
        <!--        第一列-->
        <div class="col" style="width: 90%">
          <div class="intro row">{{ plugin.intro }}</div>
          <div class="row-between">
            <span class="id">ID：{{ plugin.id }}</span>
          </div>
        </div>
        <div class="col-between" style="width: 10%;height: 90%;margin-top: auto">
          <!--          <IconBtn :size="1.2" name="edit" :color="'#da8c59'" @click="goEdit(plugin)" prompt="插件编辑"/>-->
          <IconBtn :size="1.2" name="flush" :color="'#62da59'" @click="reloadPlugin(plugin.id)"
                   :prompt="tlp('{reload}')"/>
          <IconBtn :size="1.2" name="delete" :color="'#fc6868'" @click="delPlugin(plugin.id)"
                   :prompt="tlp('{del}')"/>
        </div>
      </div>

      <div class="row-between" style="height: 20%;margin-top: auto;align-items: center">
        <div v-if="plugin.enable" class="row-between" style="width:60%;margin:auto;">
          <IconBtn v-if="plugin.webIndex" :size="1.2" name="func" :color="'#59a9da'" @click="goFunc(plugin)"
                   :prompt="tlp('{plugin} {func}')"/>
          <IconBtn :size="1.2" name="store" :color="'#b058ef'" @click="goStore(plugin)"
                   :prompt="tlp('{plugin} {stores}')"/>
          <IconBtn :size="1.2" name="log" :color="'#d5d5d5'" @click="goLog(plugin)" :prompt="tlp('{plugin} {logs}')"/>
        </div>
        <el-switch v-model="plugin.enable" @click="enablePlugin(plugin.id)" :active-value="1"
                   style="margin-left: auto"/>
      </div>
    </div>
  </div>
</template>
<script setup>

import logo from "/@/assets/logo.png";
import IconBtn from "/@/components/IconBtn.vue";
import {useRouter} from "vue-router";
import {useStore} from "vuex";
import {ref, computed} from "vue";
import {ElMessageBox} from "element-plus";
import * as api from "/@/api/api.js";
import {CONFIG} from "/@/config.js";
import Local from "/@/local.js";

const local = new Local({
  whetherDel: {zh: "是否删除此插件？", en: "Do you want to delete this plugin？"}
});
const tlp = local.tlp.bind(local);
const tlf = local.tlf.bind(local);
const tl = local.tl.bind(local);

const props = defineProps({plugin: {}})
const emits = defineEmits(['onDelete'])
const router = useRouter()
let store = useStore();

const key = ref("")

const ratio = ref(window.innerWidth / window.innerHeight)
const style = computed(() => {
  return {
    margin: (ratio.value <= CONFIG.ratio) ? '0.2rem auto' : '0.2rem 0',
    width: (ratio.value <= CONFIG.ratio) ? '90%' : '45%',
  }
})
const pLogo = computed(() => {
  if (!props.plugin.logoPath) return logo;
  if (props.plugin.logoPath.toString().startsWith("http")) return props.plugin.logoPath;
  return api.pluginLogo(props.plugin.id, props.plugin.logoPath)
})
const goEdit = (plugin) => {
  store.commit("curPlugin", plugin);
  const ratio = window.innerWidth / window.innerHeight
  if (ratio < CONFIG.ratio) {
    router.push("/sub/edit");
    return
  }
  router.push("/index/sub/edit");
}
const goLog = (plugin) => {
  store.commit("curPlugin", plugin);
  const ratio = window.innerWidth / window.innerHeight
  if (ratio < CONFIG.ratio) {
    router.push("/sub/log");
    return
  }
  router.push("/index/sub/log");
}
const goFunc = (plugin) => {
  store.commit("curPlugin", plugin);
  const ratio = window.innerWidth / window.innerHeight
  if (ratio < CONFIG.ratio) {
    router.push("/sub/func");
    return
  }
  router.push("/index/sub/func");
}
const goStore = (plugin) => {
  store.commit("curPlugin", plugin);
  const ratio = window.innerWidth / window.innerHeight
  if (ratio < CONFIG.ratio) {
    router.push("/sub/store");
    return
  }
  router.push("/index/sub/store");
}

function delPlugin(id) {

  ElMessageBox.alert(tlp("{whetherDel}？"), tlp('{warning}'), {
    confirmButtonText: tlp('{sure}'),
  }).then(_ => {
    return api.delPlugin({id}).then(_ => {
      emits("onDelete", id)
    })
  })
}

function reloadPlugin(id) {
  return api.reloadPlugin({id})
}

function enablePlugin(id) {
  return api.enablePlugin({id})
}

window.addEventListener("resize", () => {
  ratio.value = window.innerWidth / window.innerHeight
})
</script>

<style scoped lang="less">
.plugin {
  min-width: 10rem;
  height: 10rem;
  padding: 0.3rem 0.5rem;
  border-radius: 1rem;
  border: 3px #9ec6f5 solid;
  z-index: 2;
  /*margin: 0.2rem 0;*/
}

.head {
  width: 100%;
  height: 25%;
  align-items: center;
  position: relative;
  border-bottom: 2px solid rgba(255, 255, 255, 0.13)
}


.name {
  text-overflow: ellipsis;
  overflow: hidden;
  color: white;
  font-size: 1.2rem;
  width: 100%;
  position: absolute;
  margin: auto;
  align-self: flex-end;
}

.version {
  font-weight: bolder;
  color: gray;
  font-size: 10px;
  margin-left: auto;
  align-self: flex-end;
}

.intro {
  text-overflow: ellipsis;
  overflow: hidden;
  color: gray;
  font-size: 0.8rem;
  word-wrap: break-word;
  margin: 2% auto 0 auto;
  width: 98%;
  height: 80%;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 3;
  line-height: 0.8rem;
  text-align: left;
  padding: 2px;
}

.id {
  font-weight: bolder;
  color: gray;
  text-align: left;
  text-overflow: ellipsis;
  overflow: hidden;
  word-break: break-all;
  white-space: nowrap;
  margin: auto 0;
}


.logo {
  width: 1.6rem;
  height: 1.6rem;
  border-radius: 1rem;
}

</style>
