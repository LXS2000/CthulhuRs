<template>
  <div class="col" style="width: 98%;margin-left: 1%;height: 100%">
    <div id="listHead" class="row">
      <el-input
          v-model="key"
          style="width: 80%;"
          :placeholder="tl('search')"/>
      <IconBtn :size="1.2" name="search" :color="'#66cff5'"/>
    </div>
    <el-scrollbar>
      <div id="plugins">
        <Plugin v-for="plugin in plugins" :plugin="plugin" @onDelete="getPluginList"/>
      </div>
      <div style="height: 5rem"/>
    </el-scrollbar>
  </div>

</template>

<script setup>

import {onMounted, ref,} from "vue";
import IconBtn from "../../components/IconBtn.vue";
import {useRouter} from "vue-router";
import {useStore} from "vuex";

import {getPlugins} from "../../api/api.js";

import Plugin from "/@/components/Plugin.vue";
import Local from "/@/local.js";
const local = new Local({
  search: {zh: "输入关键字查询", en: "search with keyword"},
},true);
const tlp = local.tlp.bind(local);
const tlf = local.tlf.bind(local);
const tl = local.tl.bind(local);


const router = useRouter()
let store = useStore();


const key = ref("")

const plugins = ref([]);
const ratio = ref(window.innerWidth / window.innerHeight)


onMounted(() => {
  getPluginList()
})
const goEdit = (plugin) => {
  store.commit("curPlugin", plugin);
  router.push("/index/pluginEdit");
}

function getPluginList() {
  getPlugins().then(data => {
    plugins.value = data
  })
}


</script>

<style scoped>
#listHead {
  align-items: center;
  justify-content: space-around;
  width: 100%;
  height: 10%;
  margin: 1% auto;
}


#plugins {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: flex-start;
  width: 98%;
  height: 100%;
  margin: 0 auto;

}


/deep/ .el-input__wrapper {
  background-color: transparent;
}

/deep/ .el-input-group__append {
  background-color: transparent;
}

/deep/ .el-scrollbar {
  text-align: center;

}
</style>
