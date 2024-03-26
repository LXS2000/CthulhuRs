<template>
  <div class="col ">
    <div class="row" style="height: 50px;width: 100%;align-items: center;">
      <div class="row head" :style="`width:100%`">
        <div @click="goBack" class="row-between back">
          <el-icon>
            <Back/>
          </el-icon>
          <span>{{ tl('back') }}</span>
          <span>|</span>
        </div>
        <span class="row title"> {{ title }} </span>
      </div>
    </div>
    <div style="width: 100%" :style="`height:${height}px`">
      <router-view/>
    </div>
  </div>
</template>
<script setup>

import {useRouter, useRoute} from "vue-router";
import {CONFIG} from "/@/config.js";
import {computed, nextTick} from "vue";
import {useStore} from "vuex";
import {tl} from "/@/local.js";

const store = useStore();
let height = computed(() => store.state.windowHeight - 60)
let isSp = computed(() => window.innerWidth / store.state.windowHeight < CONFIG.ratio)
let route = useRoute();
let router = useRouter();

const goBack = () => {
  router.push("/index/list")
}
const title = route.meta.title;

</script>

<style scoped lang="less">
.head {
  color: #dadad8;
  height: 30px;
  margin: auto;
  align-items: flex-end;
  position: relative
}

.back {
  width: 15%;
  margin-left: 12%;
  font-weight: bold;
  align-items: center;
  //justify-content: center;
  z-index: 999998
}

.title {
  color: white;
  position: absolute;
  margin: auto;
  width: 100%;
  //left: 10%;
  font-size: 1.2rem;
  font-weight: bold;
  letter-spacing: 5px;
  justify-content: center;
  align-items: flex-end;
}

/deep/ .el-scrollbar__thumb {
  background: #ffffff;
}

/deep/ .el-scrollbar__wrap {
  height: 98%
}

/deep/ .el-scrollbar__view {
  height: 100%
}
</style>
