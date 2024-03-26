<template>
  <router-view/>
</template>
<script setup>

import {ref, provide} from "vue";

const winSize = ref({width: 0, height: 0});
const isExpand = ref(false);
window.addEventListener("message", (e) => {
  let {source, type, value} = e.data
  if (source === "cthulhu") {
    if (type === "windowExpand") {
      isExpand.value = value;
    }
    if (type === "windowResize") {
      winSize.value = value;
    }
  }
})
provide('winSize', winSize)
provide('isExpand', isExpand)


</script>
<style scoped lang="less">
::-webkit-scrollbar {
  display: none;
}

</style>
