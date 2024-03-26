<template>
  <div class="col" style="height: 100%">
    <div class="head row" style="height: 20px;">
      <div class="row" style="align-items: center">
        <IconBtn :size="1.2" name="flush" :color="'#62da59'" @click="refresh" :prompt="tl('flush')"/>
      </div>
      <div style="color: #c7c7c7;font-size: 1rem;font-weight: bold">{{ curPlugin.name }}</div>
      <div class="row" @click="openInNewTab" style="color: #5cf1f6;font-size: 0.7rem;align-items: center">{{
          tl('open')
        }}
        <el-icon>
          <CaretRight/>
        </el-icon>
      </div>
    </div>
    <el-scrollbar height="100%" :style="`height: ${height}px`">
      <iframe v-if="isShow" border="0"
              allow="clipboard-write"
              frameborder="0"
              class="page"
              :src="url"/>
      <div style="height: 3rem"/>
    </el-scrollbar>
  </div>
</template>
<script setup>
import {useStore} from "vuex";
import {computed, ref} from "vue";
import IconBtn from "/@/components/IconBtn.vue";
import Local from "/@/local.js";

const local = new Local({
  flush: {zh: "刷新插件页面", en: "flush plugin web page"},
  open: {zh: "在新页面打开", en: "open in new tab"},
}, true);
const tlp = local.tlp.bind(local);
const tlf = local.tlf.bind(local);
const tl = local.tl.bind(local);


const isShow = ref(true)

const store = useStore();
let height = computed(() => store.state.windowHeight - 80)
const curPlugin = store.state.curPlugin;
const scopeId = self["CTHULHU_SCOPE_ID"] || '';
const url = computed(() => {
  return `https://${curPlugin.id}.plugin.cthulhu.server/${curPlugin.webRoot}?scopeId=${scopeId}`
})


function openInNewTab() {
  window.open(url.value)
}

function refresh() {
  isShow.value = false;
  setTimeout(() => isShow.value = true, 200)
}
</script>
<style scoped lang="less">
.head {

  width: 100%;
  margin: auto;
  border-bottom: 1px white solid;
  justify-content: space-evenly;
  align-items: flex-end;
}

.page {
  width: 98%;
  margin: auto;
  height: 90vmax;
  border: none;
}

</style>
