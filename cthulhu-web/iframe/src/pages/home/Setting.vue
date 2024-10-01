<template>
  <div id="set-head" class="row">
    <span style="color: whitesmoke;font-weight: bolder;font-size: 1.3rem;margin: 0 auto">{{ tl("setting") }}</span>
    <!--    <IconBtn name="yes" size="1.6" :color="'#9dfc6d'" style="margin:auto 0.6rem auto 0"/>-->
  </div>
  <Struct :struct="struct" @on-updated="updated" />
</template>
<script>
export default {
  name: 'Setting'
}
</script>

<script setup>
import {onMounted, ref, watch, defineProps} from "vue";
import IconBtn from "../../components/IconBtn.vue";
import {getConfigs} from "../../api/api.js";
import Struct from "../../components/Struct.vue";
import Local from "/@/local.js";

const local = new Local({
  setting: {zh: "系统设置", en: "system setting"},
}, true);
const tlp = local.tlp.bind(local);
const tlf = local.tlf.bind(local);
const tl = local.tl.bind(local);

const struct = ref([])
const key = ref(0)
const loading = ref(false)
onMounted(() => {
  updated()
})

function updated() {
  loading.value = true;
  getConfigs().then(data => {
    struct.value = data;
    key.value++;
  }).finally(() => {
    loading.value = false;
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
  width: 80%;
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
