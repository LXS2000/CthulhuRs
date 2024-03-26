import Http from "../utils/http.js";

export const http = new Http(import.meta.env.APP_BASE_API)


export const restart = () => http.get("/server/restart", {})
export const serverInfo = (data) => http.get("/server/serverInfo", data)
export const getConfigs = (data) => http.get("/config/list", data)
export const updateConfig = (data) => http.post("/config/update", data)
export const getPlugins = (data) => http.get("/plugin/list", data)
export const makePlugin = (data) => http.post("/plugin/make", data)
export const enablePlugin = (data) => http.get("/plugin/enable", data)
export const reloadPlugin = (data) => http.get("/plugin/reload", data)
export const delPlugin = (data) => http.get("/plugin/del", data)
export const pluginLog = (data) => http.get("/plugin/log", data)
export const clearLog = (data) => http.get("/plugin/clearLog", data)
export const pluginDetail = (data) => http.get("/plugin/detail", data)
export const pluginLogo = (id,logo) => `https://${id}.plugin.cthulhu.server/${logo}`
export const treeNames = (id) => http.get("/plugin/treeNames", {id})
export const treeList = (id, tree) => http.get("/plugin/treeList", {id, tree})
