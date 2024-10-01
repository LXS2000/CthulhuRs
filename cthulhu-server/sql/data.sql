
INSERT INTO "config" ("id", "key", "parent_id", "label", "type", "value") VALUES
	(1, 'workspace', 0, '工作目录', 'str', '".\\workspace"'),
	(2, 'port', 0, '工作端口', 'num', '3000'),
	(3, 'systemProxy', 0, '设置为系统代理', 'bool', 'true'),
	(4, 'blackList', 0, '域名黑名单', 'obj', ''),
	(5, 'whiteList', 0, '域名白名单', 'obj', ''),
	(6, 'enabled', 4, '是否启用', 'bool', 'false'),
	(7, 'list', 4, '域名列表', 'list', ''),
	(8, 'enabled', 5, '是否启用', 'bool', 'true'),
	(9, 'list', 5, '域名列表', 'list', ''),
	(10, 'certificate', 0, 'CA证书', 'obj', ''),
	(11, 'key', 10, '私钥', 'str', '".\\ca\\cthulhu.key"'),
	(12, 'cert', 10, '证书', 'str', '".\\ca\\cthulhu.cer"');

