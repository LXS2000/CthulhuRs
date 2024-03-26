import axios from 'axios';

import {ElMessage} from 'element-plus';


export default class Http {
    constructor(baseURL) {
        this.baseURL = baseURL
        this.request = axios.create({
            baseURL,
            headers: {
                'Content-Type': 'application/json',
            }
        });
        // 拦截器
        this.request.interceptors.request.use(async (request) => {
            return request
        })
        this.request.interceptors.response.use((response) => {

            if (!response.headers.getContentType(/.*json.*/)) return response.data;

            const {data, code, msg} = response.data || {};
            if (code === 200) {
                if (msg)  ElMessage.success(msg)
                return data;
            }
            if (code >= 400 && code < 500 && msg) {
                ElMessage.warning(msg)
            } else if (code >= 500 && msg) {
                ElMessage.error(msg)
            }
            throw new Error(msg);
        }, async (error) => {
            if (!error.response) {
                ElMessage.error("系统异常，无响应")
                return Promise.reject(error);
            }
            ElMessage.error(error.response.toString())
            return Promise.reject(error);
        });
    }

    makeUrl(path = '/', params = {}) {
        let url = new URL(this.baseURL + path);
        Object.keys(params).forEach(key => {
            url.searchParams.set(key, params[key])
        })
        return url.toString();
    }

    get(path = '/', params = {}) {
        return this.request.request({
            url: path, method: "GET", params, headers: {
                'Content-Type': 'application/json;charset=utf-8',
            }
        })
    }

    post(path = '/', data = {}, params = {}) {
        return this.request.post(path, data, {
            headers: {
                'Content-Type': 'application/json;charset=utf-8',
            }
        })
    }

    download({path = '/', filename = '未命名文件', method = 'GET', data = {}, params = {}}) {
        return this.request.request({
            url: path,
            method,
            data,
            params,
            headers: {
                'Content-Type': 'application/json;charset=utf-8',
            }, responseType: 'blob',
        })
            .then(res => {
                downloadFromResponse(res, filename);
            })
            .catch(err => {
                console.log(err, '下载文件出错！');
                ElMessage.error('下载文件出错！');
            });
    }
}


function downloadFromResponse(response, filename) {
    if (!response) return;
    const blob = new Blob([response.data], {
        type: 'application/octet-stream;charset=UTF-8',
    });
    const downloadElement = document.createElement('a');
    const href = window.URL.createObjectURL(blob); // 创建下载的链接
    downloadElement.href = href;
    downloadElement.download = filename;
    if (!filename || filename === '未命名文件') {
        if (response.headers['Content-Disposition'] || response.headers['content-disposition']) {
            downloadElement.download = (response.headers['Content-Disposition']
                || response.headers['content-disposition']).substring(20);
        }
    }
    document.body.appendChild(downloadElement);
    downloadElement.click(); // 点击下载
    document.body.removeChild(downloadElement);
    window.URL.revokeObjectURL(href); // 释放blob对象
    ElMessage.success('下载成功');
}

