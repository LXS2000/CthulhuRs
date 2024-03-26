<!--只需要给一个dom存放svg即可-->
<template>
  <svg :class="svgClass" v-bind="$attrs"
       :style="css">
    <!--这边是做一个联动，根据传入的name来展示相关的svg-->
    <use :xlink:href="iconName"/>
  </svg>
</template>

<script setup>
import {computed, defineProps} from "vue";

let props = defineProps({
  name: String,
  color: String,
  size: Number | String,
})
let css = {
  fill: props.color,

  width: props.size + 'rem', height: props.size + 'rem'
}
// 这边就是根据props的name来进行处理啦
const iconName = computed(() => `#icon-${props.name}`);
const svgClass = computed(() => {
  if (props.name) {
    return `svg-icon icon-${props.name}`;
  }
  return "svg-icon"; // svg 标签
});

</script>


