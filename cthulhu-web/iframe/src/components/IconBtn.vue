<template>
  <div>
    <el-tooltip :disabled="!prompt" placement="top">
      <template #content>  {{ prompt }}</template>
      <SvgIcon :name="name" :color="color" :size="size"
               :class="isHover?'hover':''"
               @click="emits('click')"
               @mouseover="isHover=true"
               @mouseleave="isHover=false"
      />
    </el-tooltip>
  </div>
</template>

<script setup>
import {ref, defineProps} from "vue";
import SvgIcon from "./SvgIcon.vue";

const isHover = ref(false);
const props = defineProps({
  name: String, prompt: String,
  color: String,
  size: String | Number,
})
const emits = defineEmits(['click'])
</script>

<style scoped lang="less">
.hover {
  transform: scale(1.2);
  filter: brightness(120%);
}

/deep/ .el-popper {
  background: transparent;
  width: auto;
}

/deep/ .el-popover {
  background: transparent;
}
</style>
