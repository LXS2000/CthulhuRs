-- --------------------------------------------------------
-- 主机:                           E:\project_space\rust\CthulhuRs\cthulhu\cthulhu.db
-- 服务器版本:                        3.39.4
-- 服务器操作系统:                      
-- HeidiSQL 版本:                  12.2.0.6576
-- --------------------------------------------------------

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET NAMES  */;
/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;

-- 导出  表 cthulhu.config 结构
CREATE TABLE IF NOT EXISTS `config` (
	`id` INTEGER  PRIMARY KEY AUTOINCREMENT,
	`key` VARCHAR(50) NOT NULL DEFAULT '',
	`parent_id` TINYINT NOT NULL DEFAULT 0,
	`label` VARCHAR(50) NOT NULL DEFAULT '',
	`type` VARCHAR(5) NOT NULL, 
	`value` TEXT NOT NULL DEFAULT '',
	UNIQUE (`key`,`parent_id`)
);

-- 正在导出表  cthulhu.config 的数据：13 rows
/*!40000 ALTER TABLE "config" DISABLE KEYS */;
INSERT IGNORE INTO "config" ("id", "key", "parent_id", "label", "type", "value") VALUES
	(1, 'workspace', 0, '工作目录', 'str', '"E:\\project_space\\rust\\CthulhuRs\\cthulhu\\workspace"'),
	(2, 'port', 0, '工作端口', 'num', '3000'),
	(3, 'systemProxy', 0, '设置为系统代理', 'bool', 'true'),
	(4, 'blackList', 0, '域名黑名单', 'obj', ''),
	(5, 'whiteList', 0, '域名白名单', 'obj', ''),
	(6, 'enabled', 4, '是否启用', 'bool', 'false'),
	(7, 'list', 4, '域名列表', 'list', '[]'),
	(8, 'enabled', 5, '是否启用', 'bool', 'true'),
	(9, 'list', 5, '域名列表', 'list', '[]'),
	(10, 'certificate', 0, 'CA证书', 'obj', ''),
	(11, 'key', 10, '私钥', 'str', '"E:\\project_space\\rust\\CthulhuRs\\cthulhu\\ca\\cthulhu.key"'),
	(12, 'cert', 10, '证书', 'str', '"E:\\project_space\\rust\\CthulhuRs\\cthulhu\\ca\\cthulhu.cer"');
/*!40000 ALTER TABLE "config" ENABLE KEYS */;

/*!40103 SET TIME_ZONE=IFNULL(@OLD_TIME_ZONE, 'system') */;
/*!40101 SET SQL_MODE=IFNULL(@OLD_SQL_MODE, '') */;
/*!40014 SET FOREIGN_KEY_CHECKS=IFNULL(@OLD_FOREIGN_KEY_CHECKS, 1) */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40111 SET SQL_NOTES=IFNULL(@OLD_SQL_NOTES, 1) */;
