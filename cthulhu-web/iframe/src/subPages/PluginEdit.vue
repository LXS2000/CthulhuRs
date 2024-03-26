<template>
  <div class="fill col" style="padding: 0.2rem 0.3rem;width: 98%">
    <el-form :model="form" label-width="10rem" label-position="top">
      <el-form-item label="名称" required>
        <el-input v-model="form.name" show-word-limit maxlength="15"/>
      </el-form-item>
      <el-form-item label="Logo">
        <div class="row" style="width: 100%;align-items: center;height: 90%">
          <el-image class="logo" :src="form.logo" fit="fill">
            <template #error>
              <SvgIcon name="pictureSplit" size="1"/>
            </template>
          </el-image>
          <IconBtn name="upload" size="1.5" color="white" @click="selectFile" style="margin-left: 3rem"/>
          <input type="file" id="fileSelect" accept=".jpg,.png,.gif"/>
        </div>
      </el-form-item>
      <el-form-item label="简介">
        <el-input v-model="form.intro"
                  show-word-limit
                  maxlength="250"
                  rows="3"
                  type="textarea"/>
      </el-form-item>
      <el-form-item label="网络">
        <el-form-item label="网络监听">
          <el-switch v-model="form.netMonitor" :active-value="1" :inactive-value="0"/>
        </el-form-item>
        <el-form-item label="网络修改优先级">
          <el-input-number v-model="form.netModify" :step="1"/>
        </el-form-item>
      </el-form-item>
      <el-form-item label="域名匹配">
        <el-input v-model="form.matches"
                  placeholder="在哪些域名的网站中使用此插件，空表示所有域名，多个域名用‘;’隔开，当输入框第一个字符是‘!’表示当域名不在以下域名中时使用此插件"
                  show-word-limit
                  rows="5"
                  maxlength="500"
                  type="textarea"/>
      </el-form-item>
    </el-form>
    <el-tabs v-model="activeName">
      <el-tab-pane label="Inject" name="inject">
        <Codemirror
            :lang="lang"
            :extensions="extensions"
            basic
            style="min-height: 100px;width: 98%"
            ref="injectCode"
            v-model="form.injectCode"
            @change="">
        </Codemirror>
      </el-tab-pane>
      <el-tab-pane label="Server" name="server">
        <Codemirror
            :lang="lang"
            :extensions="extensions"
            basic
            style="min-height: 100px;width: 98%"
            ref="serverCode"
            v-model="form.serverCode"
            @change="">
        </Codemirror>
      </el-tab-pane>
    </el-tabs>
    <div style="height: 3rem"/>
  </div>
</template>

<script setup>

import Codemirror from 'vue-codemirror6'; // 引入组件
import {javascript} from "@codemirror/lang-javascript";
import {oneDark} from "@codemirror/theme-one-dark";

import {useRouter} from "vue-router";
import IconBtn from "../components/IconBtn.vue";
import SvgIcon from "../components/SvgIcon.vue";
import {useStore} from "vuex";
import {makePlugin, pluginDetail} from "../api/api.js";
import {onMounted, ref} from "vue";
import {ElMessage} from "element-plus";

let store = useStore();
const lang = javascript();

const extensions = [oneDark];

let logoFile = null;
const form = ref({
  id: undefined,
  name: '',
  intro: '',
  logo: '',
  logoName: '',
  netMonitor: false,
  netModify: 0,
  matches: '',
  injectCode: '',
  serverCode: '',
});

onMounted(() => {
  if (!(store.state.curPlugin || {}).id) {
    return
  }
  let id = store.state.curPlugin.id;

  pluginDetail({id}).then(data => {
    form.value = data;
    form.value.logo = pluginLogo(id);
  })
})

const activeName = ref('inject')
const router = useRouter();
const option = ref({
  tabSize: 2,
  mode: 'text/javascript',
  theme: 'base16-light', // 主题
  lineNumbers: true,
  line: true,
});
const goBack = () => {
  router.back()
}
const selectFile = () => {
  let fs = document.getElementById("fileSelect");
  if (!fs) return;
  return new Promise((rs, rj) => {
    fs.addEventListener('change', function () {
      // 获取单个文件
      let file = fs.files[0];
      let name = file.name;
      // 创建一个FileReader对象
      let reader = new FileReader();
      // 监听FileReader对象的load事件
      reader.addEventListener('load', function () {
        logoFile = file;
        rs({src: reader.result, name})
      });
      // 开始读取文件
      reader.readAsDataURL(file);
    });
    fs.click()
  }).then(({src, name}) => {
    form.value.logo = src;
    form.value.logoName = name;
  })
}

function onNew() {
  if (form.value.logo && form.value.logo.startsWith("data:")) {
    form.value.logo = form.value.logo.split(",")[1];
    if (!form.value.logoName) {
      ElMessage.warning("logo名称不能为空")
      return
    }
  } else {
    delete form.value.logo
    delete form.value.logoName
  }
  makePlugin(form.value).then(_ => {
    goBack()
  });
}
</script>

<style scoped lang="less">

.logo {
  min-height: 2rem;
  max-height: 3vw;
  height: 3rem;
  width: auto;
  border-radius: 1.2rem;
  //border: 0.2rem solid rgba(103, 197, 141, 0.87);
}

#fileSelect {
  display: none;
  width: 1rem;
  height: 1rem;
  background-color: white;
  color: #7ecb56;
}

/deep/ .el-form-item__label {
  color: white;
}

/deep/ .el-input__wrapper {
  background-color: transparent;
}

/deep/ .el-input__inner {
  color: #eeeeee;
  background-color: transparent;
}


/deep/ .el-textarea__inner {
  color: #eeeeee;
  background-color: transparent;
}

/deep/ .el-tabs__item {
  color: white;

}

/deep/ .el-input__count {
  background-color: transparent;
  color: white;

  .el-input__count-inner {
    background-color: transparent;
    color: white;
  }
}
</style>
