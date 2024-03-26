<template>
  <div
      :class="isHover?'hover':''"
      @mouseover="isHover=true"
      @mouseleave="isHover=false"
      :style="style"
  >
    {{ name }}
  </div>
</template>

<script setup>
import {ref, defineProps} from "vue";

const isHover = ref(false);
const props = defineProps({
  name: String,
  color: String,
  size: String | Number,
})
const style = {
  'font-size': props.size.type === String ? props.size : props.size + "px",
  'color': props.color,
  'font-weight':'bold'
}

</script>

<style scoped lang="less">
.hover {
  transform: scale(1.1);
  filter: brightness(120%);
}
</style>
