<template>
  <div class="col" style="width: 98%;height:100%;margin: 0 auto">
    <div class="row-between" style="height: 40px;width: 100%">
      <el-radio-group v-model="level" size="small" style="width: 90%;margin: 1% auto ;">
        <el-radio-button name="info" label="info"/>
        <el-radio-button name="debug" label="debug"/>
        <el-radio-button name="error" label="error"/>
        <el-radio-button name="warn" label="warn"/>
      </el-radio-group>
      <div class="row" style="width: 10%;align-items: center;margin: auto">
        <IconBtn style="margin: auto" :size="1.5" name="clear" :color="'#f6dd65'" @click="clear"
                 :prompt="tl('clear')"/>
      </div>
    </div>
    <el-scrollbar ref="scroller"
                  always id="wrap" :style="`height:${height}px`"
                  class="logView">
      <div class="col fill" style="border-radius: 0.3rem;">
          <span v-for="line in lines">
          {{ line }}
          </span>
        <div class="loadMore row">
          <div class="row" style="margin: auto">
            <span v-if="loading" class="rotate"><el-icon><Loading/></el-icon></span>
            <span v-if="!isEnd" @click="load" style="margin: auto">{{ tl('load') }}</span>
            <span v-if="isEnd" style="margin: auto">{{ tl('no') }}</span>
          </div>
        </div>
      </div>
      <div style="height: 3rem"/>
    </el-scrollbar>
  </div>
</template>

<script setup>
import {computed, onMounted, ref, watch} from "vue";
import {useRouter} from "vue-router";
import {useStore} from "vuex";
import {clearLog, pluginLog} from "../api/api.js";
import IconBtn from "/@/components/IconBtn.vue";
import Local from "/@/local.js";

const local = new Local({
  clear: {zh: "清空日志", en: "clear logs"},
  load: {zh: "点击加载", en: "click to load"},
  no: {zh: "没有日志", en: "no logs"},
}, true);
const tlp = local.tlp.bind(local);
const tlf = local.tlf.bind(local);
const tl = local.tl.bind(local);

const scroller = ref(null)
const loading = ref(false)
const isEnd = ref(false)
const level = ref("info")
const router = useRouter();
let store = useStore();
const lines = ref([]);


let height = computed(() => store.state.windowHeight - 60)

watch(level, (n, o) => {
  lines.value = []
  load()
})
onMounted(() => {
  let id = store.state.curPlugin.id
  if (!id) {
    router.push("/")
    return
  }
  load()
})

function load() {
  let id = store.state.curPlugin.id
  loading.value = true;
  let ms = 300;
  pluginLog({id, level: level.value, index: lines.value.length}).then(data => {
    if (!data.length) {
      isEnd.value = true;
      return
    }
    setTimeout(() => {
      lines.value.push(...data)
    }, ms)

  }).finally(() => {
    setTimeout(() => {
      loading.value = false;
    }, ms)
  });
}

function clear() {
  let id = store.state.curPlugin.id
  lines.value = []
  clearLog({id, level: level.value}).then(load)
}
</script>

<style scoped lang="less">
#wrap {
  width: 100%;
}

.logView {
  height: 96%;
  width: 98%;
  margin: auto;
  padding: 0.3rem;
  background-color: gray;
  border-radius: 0.3rem;
}

.rotate {
  width: 1rem;
  height: 1rem;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: auto 0;
  animation: rotate 2s linear infinite;
  -webkit-animation: rotate 2s linear infinite;
}

@keyframes rotate {
  0% {
    transform: rotate(0);
  }

  25% {
    transform: rotate(90deg);
  }

  50% {
    transform: rotate(180deg);
  }

  75% {
    transform: rotate(270deg);
  }

  100% {
    transform: rotate(360deg);
  }
}

@-webkit-keyframes rotate {
  0% {
    transform: rotate(0);
  }

  25% {
    transform: rotate(90deg);
  }

  50% {
    transform: rotate(180deg);
  }

  75% {
    transform: rotate(270deg);
  }

  100% {
    transform: rotate(360deg);
  }
}

.loadMore {
  font-weight: bold;
  color: white;
  height: 3rem;
  margin-top: 5rem;
}

/deep/ .el-radio-button {
  width: 25%;
}

/deep/ .el-radio-button__inner {
  width: 100%;
}
</style>
