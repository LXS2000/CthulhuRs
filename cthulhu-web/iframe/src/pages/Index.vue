<template>
  <div id="body" class="col" @mousedown="down">
    <div id="head" class="row">
      <div id="title">
        CthulhuRs Server
        <span style="color: white;margin-left: 0.6rem;font-size: 0.5rem">3.5.6</span>
      </div>
    </div>
    <div id="content">
      <Tabs :tabs="tabs"/>
    </div>
  </div>

</template>

<script setup>

import {onMounted, ref, watch, inject} from "vue";

import Tabs from "../components/Tabs.vue";
import Local from "/@/local.js";
const local = new Local({
  home: {zh: "主页", en: "home"},
  intro: {zh: "介绍", en: "intro"},
  plist: {zh: "插件", en: "plugins"},
  setting: {zh: "设置", en: "setting"},
},true);
const tlp = local.tlp.bind(local);
const tlf = local.tlf.bind(local);
const tl = local.tl.bind(local);


const isExpand = inject('isExpand');

const isExpandDelay = ref(false);

const tabs = ref([
  {name: tl("home"), path: "/index/home", icon: "home"},
  {name: tl("intro"), path: "/index/intro", icon: "intro"},
  {name: tl("plist"), path: "/index/list", icon: "list"},
  {name: tl("setting"), path: "/index/setting", icon: "setting"},
])

const eleExpand = (ele, isExpand) => {
  if (!ele) return;
  if (isExpand) {
    ele.classList.add("expand");
    ele.classList.remove("unexpand");
    return
  }
  ele.classList.remove("expand");
  ele.classList.add("unexpand");
}
watch(isExpand, (n, o) => {
  let head = document.getElementById("head");
  eleExpand(head, n);
  let msg = {source: "cthulhu", type: "windowExpand", value: n}
  window.top.postMessage(msg, "*")
  if (n) {
    setTimeout(() => isExpandDelay.value = true, 300)
    console.log("inject:cthulhuWindowExpand")
  } else {
    isExpandDelay.value = false
    console.log("inject:cthulhuWindowUnExpand")
  }
})
const size = {
  bw: 8, bh: 4,
  ew: 50, eh: 30
}
const unit = 1;
let offset = {x: 0, y: 0}


onMounted(() => {
  let head = document.getElementById("head");
  eleExpand(head, false);
})


const expand = (e) => {
  if (e) e.stopPropagation()
  let msg = {source: "cthulhu", type: "windowExpand", value: !isExpand.value}
  window.top.postMessage(msg, "*")
}
const down = (e) => {
  if (isExpand.value) return
  if (e) e.stopPropagation()
  let msg = {source: "cthulhu", type: "windowDown",}
  window.top.postMessage(msg, "*")
}


</script>

<style scoped lang="less">
@minh: 20px;
@minw: 40px;
@mcolor: #1e1e1e;

#body {
  overflow: hidden;
  position: absolute;
  left: 0;
  top: 0;
  background-color: transparent;
  //background-color: @mcolor;
  //opacity: 1;
  width: 100vw;
  height: 100vh;
  align-items: center;
  //transition: width 0.3s ease-in-out,
  //height 0.6s ease-in-out
}


::-webkit-scrollbar {
  display: none;
}

#more {
  font-size: 2rem;
  color: white;
  width: 2rem;
  position: absolute;
  right: 0;
  border: 0.5rem solid transparent;
}

#title {
  text-align: center;
  overflow: hidden;
  color: white;
  width: 80%;
  transition: width 0.3s ease-in-out;
}

#logo {
  min-height: 2rem;
  max-height: 3vw;
  height: 10%;
  position: absolute;
  left: 0;
  border: 0.5rem solid transparent;
}

#content {
  width: 100%;
  height: 100%;
  overflow: hidden;
}

#head {
  overflow: hidden;
  justify-content: center;
  align-items: center;
  position: relative;
  padding: 0 1vw;
  width: 98%;
  min-height: 3rem;
  height: 5vw;
  user-select: none;
  margin-bottom: 0.6rem;
}


/deep/ .el-tabs__nav {
  background-color: @mcolor;

  .el-tabs__item {
    background-color: @mcolor;
  }

  .is-active {
    background-color: @mcolor;
  }
}


</style>
