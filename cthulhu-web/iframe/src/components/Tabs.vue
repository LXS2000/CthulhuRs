<template>
  <div style="width: 100%;height: 100%" class="row">
    <div class="col tabs">
      <div v-for="(tab,i) in tabs" class="row tab"
           :id="'tab-'+i"
           @mouseenter="onHoverTab"
           @mouseleave="onLeaveTab"
           @click="onClickTab(i)">
        <span v-if="i===active" class="activeBar"/>
        <SvgIcon :size="1" :name="tab.icon" :color="tab.active?'#5cf1f6':'white'"
                 style="position: absolute;left: 20%"/>
        <span style="position: absolute;left: 50%;font-size: 0.9rem">{{ tab.name }}</span>
      </div>
    </div>
    <div style="width: 80%;padding: 0">
      <router-view/>
    </div>
  </div>
</template>

<script setup>
import {ref, defineProps, onMounted, watch} from "vue";
import {useRouter} from "vue-router";
import SvgIcon from "./SvgIcon.vue";


const router = useRouter()
const props = defineProps({
  tabs: {type: Array}
})
const tabs = ref([...props.tabs]);
const active = ref(0)

watch(active, (n, o) => {
  let tabEle = document.getElementById("tab-" + o)
  if (tabEle) tabEle.classList.remove("active")
  tabEle = document.getElementById("tab-" + n)
  if (tabEle) tabEle.classList.add("active")
  let tab = tabs.value[n];
  tab.active = true;
  tabs.value[o].active = false;
  router.push(tab.path);
})
function init(){
  if (tabs.value) {
    let tab = tabs.value[active.value];
    if (!tab) return;
    let ele = document.getElementById("tab-" + active.value)
    if (!ele) return
    ele.classList.add("active")
    router.push(tab.path)
  }
}

const onClickTab = ( i) => {
  active.value = i;
}

const onHoverTab = (e) => {
  e.stopPropagation()
  let tabEle = e.target;
  tabEle.classList.add("hover")

}
const onLeaveTab = (e) => {
  let tabEle = e.target;
  tabEle.classList.remove("hover")
}
onMounted(init)

</script>

<style scoped>
.tabs {
  width: 20%;
  align-items: center;
  height: 100%;
  position: relative;
  border-right: 1px solid white;
}

.tab {
  width: 100%;
  height: 2rem;
  /*margin: 0.2rem auto;*/
  font-size: 1rem;
  color: gray;
  mix-blend-mode: difference;
  /*background-color: #1e1e1e;*/
  position: relative;
  justify-content: center;
  align-items: center;
}

.tab.hover {
  background-color: #2a2a2a;
  color: #cecdcc;
}

.tab.active {
  background-color: #3a3a3a;
  color: #5cf1f6;
}

.activeBar {
  width: 5%;
  height: 100%;
  border-radius: 0.2rem;
  background-color: #79ffff;
  position: absolute;
  left: 0.1rem;
}
</style>
