# 名称表

FM 存档保存国家 / 赛事的数据库 ID，但不保存它们的显示名称；游戏从安装数据库渲染名称。
FM27 Scout 因此把名称表与存档解析分开。

## 国家 / 地区

内置 `shared/nation-names.csv`：

```csv
database_id,zh,en
135,克罗地亚,Croatia
```

解析时先读取 `game_db` 的 nation table，把存档内 `nation_id` 映射到 `database_id`，再用
CSV 取中文名（缺失时回退英文，再回退 `国家 #id`）。国籍筛选、球员表格和俱乐部信息都使用
这个名称。

名称表是事实性 ID → 国家名映射，不包含 SI/SEGA 的球员、俱乐部或赛事数据库内容。

## 赛事

内置 `shared/competition-names.csv`：

```csv
database_id,name
```

当前已按本地存档中出现的 `database_id` 填入 FM27 2700 数据库的赛事名称；其他存档或
数据库可在源码中继续按 `database_id` 补表，然后重新构建。赛事数据库 ID 与存档内
stage/competition ID 不同：应用从 stage 表建立赛事索引，再读取存档中成对的编辑器 ID
记录，把 competition ID 关联到 `database_id`，最后用 CSV 取名称。

停赛视图中的赛事范围停赛会显示关联后的赛事名称（未填表时回退 `Competition #id`）；
国家范围停赛会直接显示国家名称。

## 球场

内置 `shared/stadium-names.csv`：

```csv
database_id,name
680,Old Trafford
5114336,Emirates Stadium
```

这个表由 `scripts/build_stadium_names.py` 从安装的 FM27 2700 数据库第一个语言区的
`lang_db.dat` 生成（需要 `zstd` 命令）；其他数据库版本可重新生成。解析球场行时先使用
行内名称，再按球场 UID 查表。

存档的 `news`、`play_fixture_manager` 和 `edit_session_man` 还包含
`02 <球场表序号+1> <数据库 UID> <名称长度> <名称>` 形式的引用记录。解析器用它们在
存档 UID 与数据库 UID 不一致时补名称（例如 San Siro、Allianz Arena），匹配时忽略
大小写和标点。游戏清理新闻后，相关球场的引用会消失；没有行内名称、名称表记录或有效
引用的球场保持未命名，解析器不会按俱乐部或容量猜测名称。
