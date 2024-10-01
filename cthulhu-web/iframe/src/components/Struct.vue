<template>
  <el-scrollbar style="border-radius: 0.6rem">
    <div v-for="(set,i) in struct" class="item row" :key="parentKey+'-'+i">
      <div v-if="set.type!=='obj'" class="row" style="width: 100%;">
        <span v-if="level!==0" class="dot"/>
        <span class="name">{{ tl(set.key) }}</span>
        <div class="val">
          <el-input v-if="set.type==='str'" v-model="set.value" size="small" @change="onUpdate(set)"/>
          <el-input-number v-else-if="set.type==='num'" v-model="set.value" size="small" @change="onUpdate(set)"/>
          <el-switch v-else-if="set.type==='bool'" v-model="set.value" size="small" @change="onUpdate(set)"/>
          <div v-else-if="set.type==='list'" class="col list">
            <div class="row" style="margin-bottom: 0.3rem">
              <el-input v-model="set.value.input" size="small" style="width: 90%"/>
              <IconBtn name="new" size="1"
                       :color="'#6dfcfc'"
                       @click="push(set.value,set.value.input);onUpdate(set)"
                       style="margin:auto"/>
            </div>
            <el-scrollbar style="height: auto">
              <div v-for="(val,i) in set.value" class="row list-item">
                <span style="width: 5%;color: gray;font-size: 0.7rem;margin: auto">{{ i + 1 }}</span>
                <span style="width: 95%">{{ val }}</span>
                <IconBtn name="sub" size="1"
                         :color="'#f15555'"
                         @click="del(set.value,i);onUpdate(set)"
                         style="margin: auto 0.2rem auto auto"/>
              </div>
            </el-scrollbar>
          </div>
        </div>
      </div>
      <div v-else class="col" style="width: 100%;">
        <span v-if="level!==0" class="dot"/>
        <span class="name">{{  tl(set.key)  }}</span>
        <Struct :level="(level||0)+1"
                :struct="set.value"
                class="subSetting"
                @on-updated="()=>{
                  emits('on-updated')
                }"
                :style="`transform: scale(${1-((level||0)+1)*0.1})`"/>
      </div>
    </div>
    <div v-if="level===0" style="height: 5rem"/>
  </el-scrollbar>
</template>

<script setup>
import {onMounted, ref, watch, defineProps} from "vue";
import IconBtn from "../components/IconBtn.vue";
import {getConfigs, updateConfig} from "../api/api.js";
import Struct from "../components/Struct.vue";
import Local from "/@/local.js";
const local = new Local({
  workspace: {zh: "工作目录", en: "workspace directory"},
  bgColor: {zh: "背景颜色", en: "background color"},
  dbPath: {zh: "数据库文件地址", en: "db file path"},
  port: {zh: "工作端口", en: "work port"},
  systemProxy: {zh: "设置为系统代理", en: "set as system proxy"},
  enabled: {zh: "是否启用", en: "enabled"},
  list: {zh: "域名列表", en: "domain list"},
  whiteList: {zh: "域名白名单", en: "domain white list"},
  blackList: {zh: "域名黑名单", en: "domain black list"},
  certificate: {zh: "CA证书", en: "CA certificate"},
  key: {zh: "私钥", en: "key"},
  cert: {zh: "证书", en: "certificate"},
},true);
const tlp = local.tlp.bind(local);
const tlf = local.tlf.bind(local);
const tl = local.tl.bind(local);

const emits = defineEmits(["on-updated"])
const props = defineProps({
  level: Number,
  parentKey: Number,
  struct: Array
})
const level = ref(props.level || 0);


const push = (list = [], item) => {
  list.push(item)
}
const del = (list = [], i) => {
  list.splice(i, 1)
}

async function onUpdate(set) {
  let {id, value, type} = set;
  if (type === "list") {
    value = (value || []).filter(v=>!!v).join("&&")
  }
  await updateConfig({id, value}).then(res => {
    emits("on-updated")
  })
}
</script>

<style scoped lang="less">
.dot {
  width: 0.3rem;
  height: 0.3rem;
  background-color: white;
  border-radius: 0.15rem;
  margin-top: 0.5rem;
  margin-right: 0.5rem;
}

.name {
  min-width: auto;
  width: 30%;
  color: #a2a2a2;
  font-weight: bolder;
}

.val {
  min-width: auto;
  width: 70%;
}

.item {
  position: relative;
  margin: 0.5rem 1rem;
}

.list {
  width: 100%;
  //background-color: #363636;
  border-radius: 0.3rem;
  border: 1px solid gray;
  height: 8rem;
  padding: 0.1rem 0.3rem;
}

.list-item {
  color: white;
  width: 98%;
  margin: 0 auto;
  //background-color: #363636;
}

.subSetting {
  width: 100%;
}

#set-head {
  align-items: center;
  height: 2.5rem;
}

/deep/ .el-input__wrapper {
  background-color: transparent;
}

/deep/ .el-input__inner {
  color: white;
}
</style>
