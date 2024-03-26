// import axios from 'axios';
//
// import {getStore} from "./util";
// import {Message} from "element-ui";
//
// // 基础URL
//
// import {baseURL} from "./Const";
// const request = axios.create({
//   baseURL: baseURL,
//   timeout: 30000,
//   headers: {
//     'Content-Type': 'application/json',
//   },
// });
//
// // http response 拦截器
// request.interceptors.response.use(function (response) {
//   if(response.headers['loginToken']){
//     localStorage.setItem('loginToken',response.headers['loginToken'])
//   }
//   // 对返回的数据进行一些处理
//   if (response.data.code !== 200) {
//     if(response.data.message&&response.data.message.length!==0) Message.error(response.data.message)
//     return Promise.reject(response.data)
//   }
//   if (response.data.code === 200&&response.data.message&&response.data.message.length!==0) {
//     Message.success(response.data.message)
//   }
//   return response.data
// }, function (error) {
//   // 对返回的错误进行一些处理
//   const code = error.response && error.response.status;
//   if (code === 403) {
//     window.location = '#/login';
//     return
//   }
//   if (code !== 200) {
//     // 显示提示
//     Message.warning(error.response.data.message);
//   }
//   return Promise.reject(error);
// })
//
//
//
// export default request;
// /**
//  * 异步请求
//  * @param url 请求地址
//  * @param data 请求参数
//  * @param type 请求方法
//  * @param send
//  * @returns {Promise<*>}
//  */
// export const send = async (url = '', data = {}, type = 'GET') => {
//   type = type.toUpperCase();
//   if (type === 'GET') {
//     url = urlHandle(url, data);
//   }
//
//   // Get请求
//   if (type === 'GET') {
//     return request.get(url, {
//       headers: {
//         loginToken: getStore('loginToken'),
//         'Content-Type': 'application/json',
//       },
//     });
//   }
//   // Post请求
//   return request.post(url, JSON.stringify(data), {
//     headers: {
//       loginToken: getStore('loginToken'),
//       'Content-Type': 'application/json',
//     },
//   });
// };
// export const download = async (url = '', filename = '未命名文件', data = {}, type = 'GET') => {
//   type = type.toUpperCase();
//   if (type === 'GET') {
//     url = urlHandle(url, data);
//   }
//   // Get请求
//   if (type === 'GET') {
//     await request.get(url, {
//       headers: {
//         loginToken: getStore('loginToken'),
//         'Content-Type': 'application/json',
//       },
//       responseType: 'blob',
//     })
//       .then(res => {
//         downloadFromResponse(res, filename);
//       })
//       .catch(err => {
//         console.log(err, '下载文件出错！');
//         Message.error('下载文件出错！');
//       });
//   } else {
//     await request.post(url, JSON.stringify(data), {
//       headers: {
//         'Content-Type': 'application/json',
//         loginToken: getStore('loginToken'),
//       },
//       responseType: 'blob', // 二进制流
//     })
//       .then(res => {
//         downloadFromResponse(res, filename);
//       })
//       .catch(err => {
//         console.log(err, '下载文件出错！');
//         Message.error('下载文件出错！');
//       });
//   }
//
// };
//
// function urlHandle(url, data) {
//   let dataStr = '';
//   Object.keys(data)
//     .forEach(key => {
//       dataStr += `${key}=${data[key]}&`;
//     });
//   if (dataStr !== '') {
//     dataStr = dataStr.substring(0, dataStr.lastIndexOf('&'));
//     return `${url}?${dataStr}`;
//   }
//   return url;
// }
//
// function downloadFromResponse(response, filename) {
//   if (!response) return;
//   const blob = new Blob([response.data], {
//     type: 'application/octet-stream;charset=UTF-8',
//   });
//   const downloadElement = document.createElement('a');
//   const href = window.URL.createObjectURL(blob); // 创建下载的链接
//   downloadElement.href = href;
//   downloadElement.download = filename;
//   if (!filename || filename === '未命名文件') {
//     if (response.headers['Content-Disposition'] || response.headers['content-disposition']) {
//       downloadElement.download = (response.headers['Content-Disposition'] || response.headers['content-disposition']).substring(20);
//     }
//   }
//   document.body.appendChild(downloadElement);
//   downloadElement.click(); // 点击下载
//   document.body.removeChild(downloadElement);
//   window.URL.revokeObjectURL(href); // 释放blob对象
//   Message.success('下载成功');
// }
//
//
