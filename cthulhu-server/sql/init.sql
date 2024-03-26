-- 插件表
CREATE TABLE IF NOT EXISTS `plugin` (
	`id` TEXT PRIMARY KEY,
	`name` TEXT NOT NULL,
	`intro` TEXT NOT NULL,
	`version` TEXT NOT NULL,
	`path` TEXT NOT NULL ,
	`logo_path` TEXT NOT NULL DEFAULT '',
	`web_root` TEXT NOT NULL DEFAULT '',
	`web_index` TEXT NOT NULL DEFAULT '',
	`server_path` TEXT NOT NULL DEFAULT '',
	`worker_path` TEXT NOT NULL DEFAULT '',
	`context_paths` TEXT NOT NULL DEFAULT '',
	`dynamic_links` TEXT NOT NULL DEFAULT '',
	`matches` TEXT NOT NULL DEFAULT '',
	`net_monitor` INTEGER NOT NULL DEFAULT 0,
	`net_modify` INTEGER NOT NULL DEFAULT 0,
	`enable` INTEGER NOT NULL DEFAULT 0,
	`install_time` TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);


-- 配置表
CREATE TABLE `config` (
	`id` INTEGER  PRIMARY KEY AUTOINCREMENT,
	`key` VARCHAR(50) NOT NULL DEFAULT '',
	`parent_id` TINYINT NOT NULL DEFAULT 0,
	`label` VARCHAR(50) NOT NULL DEFAULT '',
	`type` VARCHAR(5) NOT NULL, 
	`value` TEXT NOT NULL DEFAULT '',
	UNIQUE (`key`,`parent_id`)
);

-- 默认
INSERT INTO "config" ("id", "key", "label", "type", "value", "parent_id") VALUES
	(1, 'workspace', '工作目录', "str", '"./"', 0),
	(2, 'dbPath', '数据库文件', "str", '"./cthulhu.db"', 0),
	(3, 'port', '工作端口', "num", '3000', 0),
	(4, 'systemProxy', '设置为系统代理', "bool", 'false', 0),
	(5, 'bgColor', '主题色', "str", '"#f8f8f8"', 0),
	(6, 'blackList', '域名黑名单', "obj", '', 0),
	(7, 'whiteList', '域名白名单', "obj", '', 0),
	(8, 'enabled', '是否启用', "bool", 'false', 6),
	(9, 'list', '域名列表', "list", '[]', 6),
	(10, 'enabled', '是否启用', "bool", 'false', 7),
	(11, 'list', '域名列表', "list", '[]', 7),
	(12, 'certificate', 'CA证书', "obj", '', 0),
	(13, 'key', '私钥', "str", '"./cthulhu.key"', 12),
	(14, 'cert', '证书', "str", '"./cthulhu.cer"', 12)
;


INSERT INTO "config" ("id", "key", "label", "type", "value", "parent_id") VALUES
	(1, 'workspace', '工作目录', "str", '"E:\\project_space\\rust\\CthulhuRs\\cthulhu\\workspace"', 0),
	(2, 'dbPath', '数据库文件', "str", '"E:\\project_space\\rust\\CthulhuRs\\cthulhu\\cthulhu.db"', 0),
	(3, 'port', '工作端口', "num", '3000', 0),
	(4, 'systemProxy', '设置为系统代理', "bool", 'false', 0),
	(5, 'bgColor', '主题色', "str", '"#f8f8f8"', 0),
	(6, 'blackList', '域名黑名单', "obj", '', 0),
	(7, 'whiteList', '域名白名单', "obj", '', 0),
	(8, 'enabled', '是否启用', "bool", 'false', 6),
	(9, 'list', '域名列表', "list", '[]', 6),
	(10, 'enabled', '是否启用', "bool", 'false', 7),
	(11, 'list', '域名列表', "list", '[]', 7),
	(12, 'certificate', 'CA证书', "obj", '', 0),
	(13, 'key', '私钥', "str", '"E:\\project_space\\rust\\CthulhuRs\\cthulhu\\ca\\cthulhu.key"', 12),
	(14, 'cert', '证书', "str", '"E:\\project_space\\rust\\CthulhuRs\\cthulhu\\ca\\cthulhu.cer"', 12)
;



