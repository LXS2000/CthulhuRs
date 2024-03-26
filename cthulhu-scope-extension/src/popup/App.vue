<template>
  <div id="index" class="col-between" style="">
    <div class="title">Cthulhu-Extra-Scope请求头编辑</div>
    <div class="col item">
      <div class="label">* Email：</div>
      <div class="intro">加入当前浏览器登录的邮箱信息</div>
      <div class="value">
        <el-switch v-model="info.email" :active-value="1" :inactive-value="0" @change="valueChangeFn('email')"/>
      </div>
    </div>
    <div class="col item">
      <div class="label">* Window：</div>
      <div class="intro">加入http请求的窗口id</div>
      <div class="value">
        <el-switch v-model="info.window" :active-value="1" :inactive-value="0" @change="valueChangeFn('window')"/>
      </div>
    </div>
    <div class="col item">
      <div class="label">* Tab：</div>
      <div class="intro">加入http请求的标签页id</div>
      <div class="value">
        <el-switch v-model="info.tab" :active-value="1" :inactive-value="0" @change="valueChangeFn('tab')"/>
      </div>
    </div>
    <div class="col item">
      <div class="label">* Frame：</div>
      <div class="intro">加入http请求的frame id（仅在v2版本的扩展中支持）</div>
      <div class="value">
        <el-switch v-model="info.frame" :active-value="1" :inactive-value="0" @change="valueChangeFn('frame')"/>
      </div>
    </div>
    <div class="col item">
      <div class="label">* Custom：</div>
      <div class="intro">加入自定义的标识符</div>
      <div class="value">
        <el-input v-model="info.custom" :maxlength="30"
                  placeholder="Please input custom tag"
                  show-word-limit
                  type="text"
                  @change="valueChangeFn('custom')"/>
      </div>
    </div>
  </div>
</template>

<script setup>
import { reactive} from "vue";
import {ElMessage} from "element-plus";

const extension = self.chrome || self.browser
const info = reactive({
  email: 0,window:0, tab: 0, frame: 0, custom: ""
});
(async () => {
  let obj = {}
  if (!extension.storage) {
    obj = localStorage.getItem("info") || "{}";
    obj = JSON.parse(obj);
  } else {
    obj = await extension.storage.local.get(["email","window", "tab", "frame", "custom"])
  }
  let {email = 0, tab = 0, window=0,frame = 0, custom = ''} = obj
  info.email = email;
  info.window = window;
  info.tab = tab;
  info.frame = frame;
  info.custom = custom;
})();

async function valueChangeFn(key) {
  if (key === 'custom') {
    if (info.custom.includes("=") || info.custom.includes(";")) {
      ElMessage.error("自定义标识符不能包含 '=' 或者';' 符号")
      return
    }
  }
  if (!extension.storage) {
    localStorage.setItem("info", JSON.stringify(info))
    return
  }
  let obj = {};
  obj[key] = info[key]
  await extension.storage.local.set(obj)
}

</script>

<style scoped>

#index {
  width: 350px;
  height: 300px;
  padding: 10px;
}

.item {
  height: 22%;
  width: 98%;
  margin: auto;
}

.title {
  font-size: 18px;
  font-weight: bold;
  color: #363636;
  text-align: center;
  width: 100%;
}

.label {
  font-size: 16px;
  font-weight: bold;
  color: #3f3a3a;
}

.intro {
  font-size: 13px;
  color: #a2a2a2;
}

.value {
  width: 96%;
  margin-left: auto;
}
</style>

