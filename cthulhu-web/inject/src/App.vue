<template>

  <div id="cthulhuWrap" class="cthulhuRow cthulhuMoveExpandAnim"
       @mouseleave="mouseleave"
       @touchleave="mouseleave">
    <img :src="logo" id="cthulhuLogo" alt="logo">
    <iframe v-if="isExpand" id="cthulhuIframe"
            border="0"
            allow="clipboard-write"
            frameborder="0"
            :src="URL"/>
    <More @mousedown="mousedown"
          @touchstart="mousedown"
          @mouseup="mouseup"
          @touchend="mouseup"
          id="cthulhuMore"/>
  </div>

</template>

<script setup>
import logo from './assets/logo.png'
import {onMounted, ref, watch} from "vue";
import More from "./More.vue";

const isExpand = ref(false)
const isHp = ref(false)
const moveAble = ref(false)
const downTime = ref(0)
const unit = 5
const lastLoc = ref({left: unit, top: unit})//竖屏时用，展开之前的位置
const size = {
  min: {width: "110px", height: "56px"},
  hp: {width: "60vw", height: "40vw"},
  sp: {width: "96vw", height: "96vh"},
}

function topx(value) {
  let units = value.replace(/[0-9.]/g, '');
  let number = parseFloat(value.replace(/[^0-9.]/g, ''));
  switch (units) {
    case 'px':
      return number;
    case 'vw':
      return number * window.innerWidth / 100;
    case 'vh':
      return number * window.innerHeight / 100;
    case 'em':
      return number * parseFloat(getComputedStyle(document.documentElement).fontSize);
    case 'rem':
      return number * parseFloat(getComputedStyle(document.body).fontSize);
      // 注意：以下单位的转换可能不准确，因为它们依赖于屏幕的 DPI 设置
    case 'cm':
      return number * 37.795275591;
    case 'mm':
      return number * 3.7795275591;
    case 'in':
      return number * 96;
    case 'pt':
      return number * 96 / 72;
    case 'pc':
      return number * 96 / 6;
    default:
      return null;
  }
}

function minimizeWrap(cthulhuWrap) {
  let {width, height} = size.min;
  width = topx(width)
  height = topx(height)
  change(cthulhuWrap, width, height)
}

function expandWrap(cthulhuWrap) {
  let width, height;
  let ratio = window.innerWidth / window.innerHeight;
  if (ratio < 1) {
    isHp.value = false;
    width = size.sp.width
    height = size.sp.height
    width = topx(width)
    height = topx(height)

  } else {
    isHp.value = true;
    width = size.hp.width
    height = size.hp.height
    width = Math.max(topx(width), 700)
    height = Math.max(topx(height), 460)
  }
  change(cthulhuWrap, width, height)
}

const change = (cthulhuWrap, width, height,) => {
  let rect = cthulhuWrap.getBoundingClientRect();
  let {left, top, width: curWidth, height: curHeight} = rect
  let ratio = window.innerWidth / window.innerHeight;
  //竖屏
  if (ratio < 1) {
    cthulhuWrap.style.width = width + "px";
    cthulhuWrap.style.height = height + "px";
    if (isExpand.value) {
      //记录展开前的位置
      lastLoc.value.left = left
      lastLoc.value.top = top
      //直接全屏居中
      cthulhuWrap.style.left = (window.innerWidth - width) / 2 + 'px'
      cthulhuWrap.style.top = (window.innerHeight - height) / 2 + 'px'
    } else {
      cthulhuWrap.style.left = lastLoc.value.left + 'px'
      cthulhuWrap.style.top = lastLoc.value.top + 'px'
    }
    return;
  }
  //横屏
  if (left <= 0) left = unit
  if (top <= 0) top = unit

  let right = window.innerWidth - (curWidth + left)
  let bottom = window.innerHeight - (Math.max(curHeight, height) + top)

  if (left < right) cthulhuWrap.style.left = unit + 'px';
  else cthulhuWrap.style.left = (window.innerWidth - (unit + width)) + 'px';

  if (bottom < 0) cthulhuWrap.style.top = (window.innerHeight - (height + unit)) + 'px';
  cthulhuWrap.style.width = width + "px";
  cthulhuWrap.style.height = height + "px";

}

watch(isExpand, (n, o) => {
  const cthulhuWrap = document.getElementById("cthulhuWrap");
  reset(cthulhuWrap)
  cthulhuWrap.classList.add("cthulhuMoveExpandAnim")
  if (n) {
    expandWrap(cthulhuWrap)
  } else {
    minimizeWrap(cthulhuWrap)
  }
})



const attach = (ele) => {
  if (!ele) return
  const {left, width} = ele.getBoundingClientRect();
  let right = window.innerWidth - (width + left)
  if (left < right) ele.style.left = unit + 'px';
  else ele.style.left = (window.innerWidth - (unit + width)) + 'px';
}
const reset = (ele) => {
  ele.classList.remove("cthulhuMoveExpandAnim")
  ele.classList.remove("cthulhuMoveAnim")
  ele.classList.remove("cthulhuExpandAnim")
  ele.classList.remove("cthulhuFocus")
}

const mousedown = (e) => {
  if (isExpand.value && downTime.value) return
  e.stopPropagation();
  e.preventDefault();
  downTime.value = new Date().getTime();
  //如果是竖屏并且已经展开 返回
  if (!isHp.value && isExpand.value) return;
  setTimeout(() => {
    let now = new Date().getTime()
    ///长按超过500毫秒 聚焦可移动
    if (now - downTime.value > 500 && downTime.value) {
      const cthulhuWrap = document.getElementById("cthulhuWrap");
      reset(cthulhuWrap)
      cthulhuWrap.classList.add("cthulhuFocus")
      downTime.value = 0;
      moveAble.value = true;
    }
  }, 501);
}

const onAttach = (e) => {
  if (moveAble.value) {
    console.log('onAttach')
    //终止长按 取消聚焦
    downTime.value = 0;
    moveAble.value = false;
    const cthulhuWrap = document.getElementById("cthulhuWrap");
    reset(cthulhuWrap);
    cthulhuWrap.classList.add("cthulhuMoveExpandAnim")
    attach(cthulhuWrap)
  }
}


const mouseup = (e) => {
  e.stopPropagation();
  e.preventDefault();

  ///鼠标按下到抬起小于500毫秒视为点击 则展开
  if (downTime.value && new Date().getTime() - downTime.value < 500) {
    if (!isExpand.value) {
      setTimeout(() => {
        isExpand.value = true
      }, 300)
    } else isExpand.value = false;
    downTime.value = 0;
    return
  }
  //长按取消
  // downTime.value = 0;
  // moveAble.value = false;
  const cthulhuWrap = document.getElementById("cthulhuWrap");
  cthulhuWrap.classList.remove("cthulhuFocus")
  onAttach()
}

window.addEventListener("mouseup",mouseup)
window.addEventListener("touchend",mouseup)
const mouseleave = (e) => {
  let time = downTime.value
  if (time !== 0) {
    setTimeout(() => {
      if (time !== downTime.value) return
      moveAble.value = false;
      downTime.value = 0;
      const cthulhuWrap = document.getElementById("cthulhuWrap");
      cthulhuWrap.classList.remove("cthulhuFocus")
      cthulhuWrap.classList.add("cthulhuExpandAnim")
      cthulhuWrap.classList.add("cthulhuMoveAnim")

    }, 1000)
  }
}

window.addEventListener('resize', () => {
  const cthulhuWrap = document.getElementById("cthulhuWrap");
  if (isExpand.value) {
    expandWrap(cthulhuWrap)
  } else {
    reset(cthulhuWrap);
    cthulhuWrap.classList.add("cthulhuMoveExpandAnim")
    attach(cthulhuWrap)
  }
  //当窗口太小直接 缩小
  if (window.innerWidth < 360 || window.innerHeight < 360) {
    isExpand.value = false;
  }
});

function onMove(e) {
  if (!(moveAble.value)) return
  const cthulhuWrap = document.getElementById("cthulhuWrap");
  let x = e.clientX || e.targetTouches[0].clientX; // 获取鼠标的X坐标
  let y = e.clientY || e.targetTouches[0].clientY; // 获取鼠标的Y坐标
  // debugger;
  // 获取元素的宽度和高度
  let elementWidth = cthulhuWrap.offsetWidth;
  let elementHeight = cthulhuWrap.offsetHeight;
  //使拖拽点固定在右上角
  x -= elementWidth

  // 获取浏览器的内部视图的宽度和高度
  let windowWidth = window.innerWidth;
  let windowHeight = window.innerHeight;

  // 确保元素不会超出浏览器的内部视图
  if (x + elementWidth > windowWidth) {
    x = windowWidth - elementWidth - unit;
  }
  if (y + elementHeight + unit > windowHeight) {
    y = windowHeight - elementHeight - unit;
  }
  if (x < unit) x = unit;
  if (y < unit) y = unit;

  // 设置元素的left和top属性
  cthulhuWrap.style.left = x + 'px';
  cthulhuWrap.style.top = y + 'px';
}

window.addEventListener("mousemove",onMove)
window.addEventListener("touchmove",onMove)


//初始化
isExpand.value = false;
onMounted(() => {
  const cthulhuWrap = document.getElementById("cthulhuWrap");
  cthulhuWrap.style.left = unit + "px";
  cthulhuWrap.style.top = unit + "px";
  minimizeWrap(cthulhuWrap)
})


let scopeId = self["CTHULHU_SCOPE_ID"] || '';
let url = "https://web.cthulhu.server/iframe/index.html"
// url = "http://localhost:889/"
const URL = ref(url + (scopeId ? '?scopeId=' + scopeId : ''))

</script>

<style scoped>
.cthulhuRow {
  display: flex;
  flex-direction: row;
}

#cthulhuWrap {
  overflow: hidden;
  z-index: 99999;
  position: fixed;
  border-radius: 25px;
  max-width: 100vw;
  max-height: 100vh;
  justify-content: center;
  background: rgba(30, 30, 30, 0.93);
}


#cthulhuWrap.cthulhuFocus {
  box-shadow: 0 0 3px 2px #bbbbbb;
  transform: scale(1.01);
}

#cthulhuIframe {
  position: absolute;
  height: 100%;
  width: 100%;
  border: none;
  margin: auto;
  z-index: 999991;
}

#cthulhuLogo {
  height: 38px;
  width: 38px;
  margin: 9px auto 9px 5px;
  object-fit: cover;
  z-index: 999992;
}

#cthulhuMore {
  fill: white;
  width: 40px;
  margin: 8px 5px 8px auto;
  height: 40px;
  z-index: 999992;
}

#cthulhuWrap.cthulhuExpandAnim {
  transition: width 0.6s ease-in-out,
  height 0.6s ease-in-out
}

#cthulhuWrap.cthulhuMoveAnim {
  transition: left 0.6s ease-in-out, top 0.6s ease-in-out;
}

#cthulhuWrap.cthulhuMoveExpandAnim {
  transition: width 0.6s ease-in-out,
  height 0.6s ease-in-out, left 0.6s ease-in-out, top 0.6s ease-in-out;
}

</style>
