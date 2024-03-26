import axios from 'axios';
import {baseURL} from './Const'
import request from "@/utils/request";
import {ElMessage} from 'element-plus'
import router from "@/popup/router";

// 基础路径
const instance = axios.create({
    baseURL,
    timeout: 30000,
    headers: {
        'Content-Type': 'application/json',
    },
});

// use(两个参数)


// 添加响应拦截器
instance.interceptors.response.use(function (response) {

    // 对返回的数据进行一些处理
    if (response.data?.code === 403) {
        router.push('/login')
        // window.location = '#/login';
        return
    }
    if (response.data?.code !== 200) {
        if (response.data.message && response.data.message.length !== 0) {
            // 显示提示
            ElMessage({
                message: response.data.message,
                type: 'warning',
            })
        }
        return Promise.reject(response.data?.message)
    }
    if (response.data.code === 200 && response.data.message && response.data.message.length !== 0) {
        ElMessage({
            message: response.data.message,
            type: 'success',
        })
    }
    return response.data?.data
}, function (error) {
    // 对返回的错误进行一些处理
    const code = error.response?.status;
    if (code === 403) {
        router.push('/login')
        return
    }
    if (code !== 200) {
        if(!!!error.response.data.message){
            error.response.data.message="系统异常"
        }
        // 显示提示
        ElMessage({
            message: error.response.data.message,
            type: 'warning',
        })

    }
    return Promise.reject(error);
})


// get请求
export const get = (url, arg) => {
    let loginToken = localStorage.getItem("loginToken")
    return instance.get(url, {params: arg, headers: {loginToken}});
}

// post请求
export const post = (url, arg) => {
    // 包含baseURL字段时，替换默认的请求根域名
    let loginToken = localStorage.getItem("loginToken")
    return instance.post(url, arg, {headers: {loginToken}});
}

export default instance;
// "content_scripts": [
//     {
//         "matches": ["<all_urls>"],
//         "js": [
//             "js/content.js"
//         ],
//
//         "run_at": "document_start"
//     }
// ],