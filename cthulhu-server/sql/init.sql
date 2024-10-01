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
CREATE TABLE IF NOT EXISTS `config` (
	`id` INTEGER  PRIMARY KEY AUTOINCREMENT,
	`key` VARCHAR(50) NOT NULL DEFAULT '',
	`parent_id` TINYINT NOT NULL DEFAULT 0,
	`label` VARCHAR(50) NOT NULL DEFAULT '',
	`type` VARCHAR(5) NOT NULL, 
	`value` TEXT NOT NULL DEFAULT '',
	UNIQUE (`key`,`parent_id`)
);

