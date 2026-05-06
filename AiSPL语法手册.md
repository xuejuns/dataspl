### **AiSPL语法手册**
**AiSPL** (AI Search Processing Language) 是飞梭数据中台专为日志数据分析设计的处理语言，支持多级管道处理流程。原始数据首先通过**索引过滤条件**进行初步筛选，再通过**多级SPL指令**进行结构化提取、字段操作和数据加工，最终输出处理结果。

#### **一、**数据源指定

```spl
'索引名模式'
```

**支持数据源**

- 原始日志：`'ailpha-securitylog-*'`
- 原始告警：`'ailpha-securityalarm-*'`
- 原始告警：`'ailpha-securityevent-*'`

#### **二、基础分析**

**数据过滤**

```spl
 filter 字段名 运算符 值
```
- 等值过滤：` filter severity == 5`
- 范围过滤：`filter destPort > 1024`
- 时间过滤：` filter collectorReceiptTime >= "2025-07-18 01:00:00"`
- 字段存在性：` filter srcAddress exist`

**语法说明**

| 语法         | 描述       | 示例                                                         | 适配字段类型             |
| :----------- | :--------- | ------------------------------------------------------------ | ------------------------ |
| AND          | 与         | srcAddress == "192.168.1.101" AND destAddress == "192.168.1.102" | 所有                     |
| OR           | 或         | srcAddress =="192.168.1.101" OR destAddress == "192.168.1.102" | 所有                     |
| NOT          | 非         | NOT (srcAddress == "192.168.1.101")                          | 所有                     |
| ==           | 等于       | srcAddress == "192.168.1.101" <br />srcAddress == "192.168.10.*" <br />srcAddress == "192.168.10.100/24" <br />srcAddress == "fe80:*" <br />srcAddress == "fe80::215:17ff:fe11:31a1/124"<br /><br />srcAddress == "10.50.86.197-10.50.86.199"<br />srcAddress == destAddress（同类型字段值比较）<br />isAPT == true（boolean类型）<br />destPort == 1024（数字类型） | 除array                  |
| !=           | 不等于     | srcAddress != "192.168.1.101" <br />srcAddress != "192.168.10.*" <br />srcAddress != "192.168.10.100/24" <br />srcAddress != "fe80:*" <br />srcAddress != "fe80::215:17ff:fe11:31a1/124"<br />srcAddress != "10.50.86.197-10.50.86.199"<br />srcAddress  != destAddress（同类型字段值比较）<br />isAPT != true（boolean类型）<br />destPort != 1024（数字类型） | 除array                  |
| exist        | 存在       | destAddress exist                                            | 所有                     |
| notexist     | 不存在     | destAddress notexist                                         | 所有                     |
| in           | 属于       | catOutcome in ["OK","FAIL"]                                  | enum                     |
| notin        | 不属于     | catOutcome notin  ["OK","FAIL"]                              | enum                     |
| startwith    | 开始于     | name startwith "远程"                                        | String                   |
| endwith      | 结束于     | requestUrl endwith ".php"                                    | String                   |
| >            | 大于       | destPort > 1024                                              | Double、float、int、long |
| <            | 小于       | destPort < 1024                                              | Double、float、int、long |
| >=           | 大于等于   | destPort >= 1024                                             | Double、float、int、long |
| <=           | 小于等于   | destPort <=1024                                              | Double、float、int、long |
| contains     | 包含       | message contains "SQL注入"                                   | String、array            |
| notcontains  | 不包含     | name notcontains "弱口令"                                    | String、array            |
| NOT contains | 不包含     | NOT name contains "弱口令"                                   | string                   |
| =~           | 正则匹配   | message =~ /\d{2}/                                           | string                   |
| !~           | 正则不匹配 | message !~ /\d{2}/                                           | string                   |

---
注意：在正则匹配的时候，不支持不区分大小写的方式

#### **三、聚合分析**
```spl
 summarize 聚合函数(字段) as 别名 by 分组字段
```
- **基础聚合结构**

  **基础聚合：**按指定字段（srcAddress）分组统计

  ```spl
  summarize COUNT() as request_count by srcAddress
  ```



**多聚合操作：**  按来源地址（srcAddress）、目标地址（destAddress）进行组合分组统计选择时间内访问请求次数（cnt）和流入数据的平均字节数（avgBytesIn）

  ```spl
  summarize COUNT(srcAddress) as cnt, AVG(bytesIn) as avgBytesIn by srcAddress,destAddress
  ```



**时序聚合：**按1分钟时间窗口（collectorReceiptTime）和源地址（srcAddress）分组，收集每个分组内的目标地址列表（victim）。

  ```spl
  summarize collect(destAddress) as victim by bin collectorReceiptTime interval 1m,srcAddress
  ```

- **聚合操作类型**

  | 语法            | 示例                                                         | 作用                             |
    | :-------------- | :----------------------------------------------------------- | :------------------------------- |
  | COUNT()         | summarize COUNT() as total_events by srcAddress              | 统计所有行的数量（包括NULL值行） |
  | COUNT(field)    | summarize COUNT(destPort) as port_hits by srcAddress         | 统计指定字段中非NULL值的数量     |
  | DCOUNT(field)   | summarize DCOUNT(destAddress) as unique_targets by srcAddress | 统计指定字段中去重后的唯一值数量 |
  | SUM(number)     | summarize SUM(bytesIn) as total_inbound by srcAddress        | 计算数值字段的总和               |
  | AVG(number)     | summarize AVG(responseTime) as avg_latency by srcAddress     | 计算数值字段的平均值             |
  | MAX(number)     | summarize MAX(bytesIn) as max_inbound by srcAddress          | 返回字段的最大值                 |
  | MIN(number)     | summarize MIN(bytesIn) as min_inbound by srcAddress          | 返回字段的最小值                 |
  | STDDEV(number)  | summarize STDDEV(responseTime) as latency_stddev by srcAddress | 计算数值字段的标准差             |
  | COLLECT(string) | summarize COLLECT(destAddress) as target_list by srcAddress  | 将字段的所有值收集到一个列表中   |

- **分组字段语法**

  | 语法                 | 示例                                    | 作用                 |
    | :------------------- | :-------------------------------------- | -------------------- |
  | by 字段名            | by srcAddress                           | 按单个字段值分组     |
  | by 字段1, 字段2, ... | by srcAddress, destPort                 | 按多个字段组合值分组 |
  | by 时间函数 时间字段 | by bin collectorReceiptTime interval 1m | 按时间函数结果分组   |

- **时序聚合详细语法**

  | 语法                         | 示例                                 | 输出格式     |
    | :--------------------------- | :----------------------------------- | :----------- |
  | bin 时间字段 interval n 单位 | bin collectorReceiptTime interval 5m | 固定窗口     |
  | s(秒), m(分), h(时), d(天)   | interval 1h<br />interval 30s        | 时间间隔单位 |

---

注意:在分组字段中使用bin函数时，每个分析的结果后自动多2个字段，分别为`_wStart`和`_wEnd`，用于表示分组的开始时间和结束时间。

#### 四、字段操作

```spl
 let 新字段名 = 表达式
```
- **字段赋值**

  - 将字段requestHeader值赋值给reqHeader ，如果已有 `reqHeader` 字段会被覆盖：

    let reqHeader = requestHeader

- **字段运算**

  - 加法： let  abc =  bytesIn + bytesOut
  - 减法： let  abc =  bytesIn -  bytesOut
  - 乘法： let  abc =  bytesIn *  bytesOut
  - 除法： let  abc =  bytesIn /  bytesOut
  - 四则运算：let  abc = (bytesIn-bytesOut)*2

- **函数调用**

  - 当 logType 字段为空时，使用默认值 "flow"，并将结果命名为abc：

    let  abc = ifnull(logType,\"flow")



---

#### **五、活动列表碰撞（开发中）**

```spl
hit 活动列表名 字段1, 字段2,... ON 列表键 where 列表过滤条件 RETURN 列表字段1 as 别名, 列表字段2 as 别名,...
```

**语法说明**

| 参数           | 必填 | 说明                                                         | 示例                   |
| :------------- | :--- | :----------------------------------------------------------- | :--------------------- |
| **活动列表名** | 是   | 列表类型为元素列表的活动列表ID                               | DX_enrich              |
| **碰撞字段**   | 是   | 碰撞数据源字段，多个字段用逗号分隔（不支持类型：boolean、enum、timestamp） | destAddress,srcAddress |
| **ON 列表键**  | 是   | 活动列表比对字段，支持多字段。根据字段匹配方式进行匹配。     | deviceIP,assetIP       |
| **where 条件** | 否   | 维表过滤条件，用于筛选活动列表数据（语法同filter）           | fileName=="fil21"      |
| **RETURN**     | 是   | 返回的维表字段及别名，格式为`字段 as 别名`                   | assetCode as myname1   |

**示例**

```spl
hit DX_enrich destAddress,srcAddress ON deviceIP,assetIP where fileName=="fil21" RETURN assetCode as myname1,assetName as myname2
```

---



#### **六、维表关联**（开发中）

```spl
 dbload 维表别名 "JDBC连接参数" 
|lookup 维表别名 主表字段 join 维表关联键 return 维表字段1 as 别名1, 维表字段2 as 别名2, ...
```
**1. JDBC连接参数格式**

```spl
"url=数据库URL>,user=<用户名>,password=<密码>,schema=<模式名>,tableName=<表名>"
```

- **支持数据库类型**

  **PostgreSQL**：

  ```spl
  dbload mydim "url=jdbc:postgresql://10.20.183.11:45432/test,user=dbapp,password=xxx,schema=public,tableName=tbl_asset"
  ```

  **MySQL：**

  ```spl
  dbload mydim "url=jdbc:mysql://localhost:3306/flink_web,user=root,password=root123456,tableName=dim_method"
  ```


**2. lookup 关联语法**

```spl
lookup 维表别名 维表关联键 join 主表字段 return 维表字段1 as 别名1, 维表字段2 as 别名2, ...
```

**示例**

```spl
lookup mydim deviceSendProductName join device_name return org_name  as device_org, org_owner as device_leader,is_deleted
```

---

#### **七、时间语法**
```spl
| let last3d = now()-3d        # 表示当前时间减去3天
| filter collectorReceiptTime>last3d     # 过滤近3天的数据
```
时间单位包括:d(天),h(小时),m(分钟),s(秒)
在进行时间过滤的时候尽量使用这种写法，速度会比较快

---

#### **八、字段投影**
```spl
| project 字段1, 字段2...       # 保留字段（只输出字段1，字段2）
| project -字段1, -字段2...     # 排除字段（不输出字段1，字段2）
```

---

#### **九、窗口分析**
```spl
 windowsummarize 聚合函数(字段) as 别名 by 分组字段 window 窗口大小 on 时间字段
```
**示例**

按照5分钟的时间窗口（基于collectorReceiptTime时间字段）和源地址（srcAddress）进行分组，计算每个分组内bytesIn的平均值，并将结果命名为avg_in。

```spl
windowsummarize avg(bytesIn) as avg_in by srcAddress window 5m on collectorReceiptTime
```

---

#### **十、排序与限制**
```spl
| sort +字段名    # 按指定字段升序排列
| sort -字段名   # 按指定字段降序排列  
| sort +字段1, -字段2...   # 先按字段1升序，再按字段2降序
| limit N        # 限制返回N条记录
```

---

#### **十一、数据输出**
```spl
| output 结果表名      # 持久化存储  （用于后续关联）  
```

---

#### **十二、内置函数**

##### 1. 字符串处理

| 函数                       | 示例                                   | 作用                                                         |
| :------------------------- | :------------------------------------- | ------------------------------------------------------------ |
| concat("str1","str2",...)` | let abc = concat("ip地址:",srcAddress) | 拼接多个字符串                                               |
| substring(field,start,len) | let subStr = substring(name,1,3)       | 截取字符串，start(number): 起始位置, end(number): 结束位置(可选) |
| trim(string)               | let trimVal = trim(name)               | 去除字符串两端空格                                           |
| lower(string)              | let lowerVal = lower(name)             | 将字符串转换成小写字符串                                     |
| upper(string)              | let upperVal = upper(name)             | 将字符串转换成大写字符串                                     |
| strlen(string)             | let strLen = strlen(name)              | 获取字符串长度                                               |
| md5(string)                | let hash1 = md5(destHostName)          | 计算字符串的MD5哈希值                                        |

##### 2. 时间处理

| 函数                          | 示例                                                   | 作用                                                         |
| :---------------------------- | :----------------------------------------------------- | ------------------------------------------------------------ |
| currentDate()                 | let abc = currentDate()                                | 获取当前日期                                                 |
| currentTime()                 | let abc = currentTime()                                | 获取当前时间                                                 |
| now()                         | let nowTime = now()                                    | 获取当前系统时间                                             |
| dateFormat(timestamp, format) | let abc = dateFormat(now(),"yyyy-MM-dd HH:mm:ss")      | 格式化时间字符串                                             |
| year(timestamp)               | let yearVal = year(collectorReceiptTime)               | 提取时间字段的年份（如 2025）                                |
| quarter(timestamp)            | let quarterVal = quarter(collectorReceiptTime)         | 提取时间字段的季度（1 到 4）                                 |
| month(timestamp)              | let monthVal = month(collectorReceiptTime)             | 提取时间字段的月份（1 到 12）                                |
| week(timestamp)               | let weekVal = week(collectorReceiptTime)               | 提取时间字段在一年中的周数（ISO 8601标准，范围 1 到 53）     |
| day(timestamp)                | let dayVal = day(collectorReceiptTime)                 | 提取时间字段在月份中的日期（1 到 31）                        |
| hour(timestamp)               | let hourVal = hour(collectorReceiptTime)               | 提取时间字段的小时部分（0 到 23）                            |
| minute(timestamp)             | let minuteVal = minute(collectorReceiptTime)           | 提取时间字段的分钟部分（0 到 59）                            |
| second(timestamp)             | let secondVal = second(collectorReceiptTime)           | 提取时间字段的秒数（0 到 59）                                |
| toDate(string)                | let dateVal = toDate(mytime)                           | 将字符串转为日期类型，格式为“yyyy-MM-dd”                     |
| toTimestamp(string)           | let timestampVal = toTimestamp(mytime)                 | 将字符串转换为完整的时间戳（Timestamp）类型，格式为“yyyy-MM-dd HH:mm:ss[.SSS]” |
| timestampToInteger(timestamp) | let intTime = timestampToInteger(collectorReceiptTime) | 时间戳转整数（毫秒）                                         |
| integerToTimestamp(int)       | let ts = integerToTimestamp(1633046400000)             | 整数转时间戳                                                 |

##### 3. JSON处理

| 函数                        | 示例                                     | 作用                 |
| :-------------------------- | :--------------------------------------- | -------------------- |
| jsonValue(field, "$.path")  | let abc = jsonValue(name, "$.device_id") | 提取JSON元素         |
| jsonExists(field, "$.path") | let abc = jsonExists(name, "$.token")    | 检查JSON路径是否存在 |
| isJson(field)               | let abc = isJson(rawEvent)               | 检查是否是JSON       |

##### 4. 类型转换

| 函数             | 示例                               | 作用       |
| :--------------- | :--------------------------------- | ---------- |
| tointeger(field) | let intVal = tointeger(rawEvent)   | 转整数     |
| toDouble(string) | let doubleVal = toDouble(rawEvent) | 转双精度数 |
| toString(number) | let strVal = toString(bytesIn)     | 转字符串   |

##### 5.数学函数

| 函数         | 示例                                        | 作用                     |
| :----------- | :------------------------------------------ | ------------------------ |
| sqrt(double) | let douVal = sqrt(bytesIn)                  | 计算平方根               |
| sqrt(double) | let dou = sqrt(toDouble(toString(bytesIn))) | 计算平方根（需转换类型） |

##### 6. 条件逻辑

| 函数                      | 示例                                        | 作用     |
| :------------------------ | :------------------------------------------ | -------- |
| if(condition, true,false) | let conditionVal = if(bytesIn>100,1,0)      | 条件判断 |
| ifnull(field, default)    | let ifNullVal = ifnull(logType,\"unknown\") | 空值替换 |

##### 7. 数组函数

| 函数                      | 示例                                       | 作用     |
| :------------------------ | :-----------------------------------------| -------- |
| array_count				| let arrayCnt = array_count(tags)			|计算array字段中的元素个数
| array_distinct			| let arrayDst = array_distinct(tags)		|对array字段中的元素进行去重
| array_first				| let arrayFst = array_first(tags)			|取array数组中的第一个元素
| array_last				| let arrayLst = array_last(tags)			|取array数组中的最后一个元素
| array_get					| let arrayGet = array_get(tags,3)			|取array数组中指定的第几个元素
| array_max					| let arrayMax = array_max(tags)			|取array数组中最大的元素
| array_min					| let arrayMin = array_min(tags)			|取array数组中最小的元素
| array_sort				| let arraySrt = array_sort(tags)			|对array数组中的元素进行正序排序
| array_reverse				| let arrayRvs = array_reverse(tags)		|对array数组中的顺序进行逆序
| array_sub					| let arraySub = array_sub(tags,3,6)		|对array数组中的元素进行切块
| array_intersect			| let arrayInt = array_inersect(tags1,tags2)|对2个数组求交
| array_union				| let arrayUni = array_union(tags1,tags2)	|对2个数组求交并
| array_except				| let arrayExp = array_except(tags1,tags2)	|对2个数组求差
| array_join				| let arrayJoi = array_join(tags,",")		|将一个数组通过分隔符拼接成一个字符串
| toArray					| let arrayTo = toArray(tags,",")			|将字符串通过，分隔形成一个数组

---

#### **十二、复杂分析示例**
```spl
filter catBehavior == "/Authentication/Verify" AND catOutcome == "OK" AND srcUserName exist
| summarize dcount(srcGeoCity) as dcnt by bin collectorReceiptTime interval 1d,srcUserName
| filter dcnt>3
| output abcd
| 'ailpha-securityalarm-*'
| filter startTime >= "2025-10-09 00:00:00" and startTime < "2025-10-11 00:00:00" and requestUrl exist
| summarize count(*) as cnt by bin startTime interval 1h,requestUrl
| windowsummarize avg(cnt) as myavg by requestUrl window 36h on startTime
| windowsummarize stddev(myavg) as mystddev by requestUrl window 36h on startTime
| filter myavg > 1 and cnt > (myavg + mystddev* 1)
| lookup abcd srcUserName join srcUserName return _wStart as start1
| filter mystddev exist and start1 > _wStart
| limit 10
```
**流程说明**：

1. 筛选条件：catBehavior == "/Authentication/Verify" AND catOutcome == "OK" AND srcUserName exist
2. 按天（1天间隔）和srcUserName分组，统计每个分组中srcGeoCity的唯一值数量（去重计数），记为dcnt。
3. 筛选上述统计结果中dcnt大于3的记录。
4. 将结果输出到名为abcd的临时表或变量中。
5. 查询原始告警数据源
6. 筛选条件：startTime 在2025-10-09 00:00:00到2025-10-11 00:00:00之间，且requestUrl存在。
7. 按小时（1小时间隔）和requestUrl分组，计算每组的记录数，记为cnt。
8. 按requestUrl分组，在36小时的窗口内（基于startTime）计算cnt的平均值，记为myavg。
9. 按requestUrl分组，在36小时的窗口内（基于startTime）计算myavg的标准差，记为mystddev。
10. 筛选条件：myavg大于1且cnt大于（myavg加上mystddev乘以1）的值。
11. 与之前输出的abcd表进行连接，连接键为srcUserName，并返回abcd表中的_wStart字段（重命名为start1）。
12. 筛选条件：mystddev存在且start1大于当前行的_wStart（当前窗口的开始时间）。
13. 限制输出10条记录。

---

> **说明**：AiSPL语法采用管道式设计（`|` 分隔操作），支持链式组合复杂分析流程，适用于实时监控、日志分析等场景。

#### **十三、注意事项**
1，在正则表达时中，如果有|字符，需要添加转义符\|
2，在正则表达式中，不要使用忽略大小写的写法/i和全局搜索/g
3，在let语句中，设置的变量名称不要跟原有的字段名称一样，原有的字段有如下:
deviceVersion
severity
attackerSecurityZone
srcGeoCounty
srcGeoCountryCode
destGeoCity
destGeoAddress
txId
srcGeoCountry
srcGeoCity
ldapOpCode
rpcXid
rdpTrafficType
destDnsDomain
ruleId
incidentId
srcDnsDomain
srcAssetId
shutdownTrue
PcapFileUniqueId
flowAlerted
rpcStatus
sessionId
flowMaxTTL
kerberosMsgType
collectorReceiptTime
databaseName
ruleName
dataSubType
logType
securityEyeLogType
flowPackets
flowSrcPort
flowSrcAddress
nfsProcedure
ldapSearch
catSignificance
ccUserName
catTechnique
srcGeoAddress
company
product
wmiFilter
flowDestAddress
cost
catBehavior
catOutcome
wmiConsumer
wmiType
wmiDestination
routerId
dhcpAddress
errorInfo
executeResult
tcpBytes
bytesIn
timesTill
flowDirection
wmiQuery
wmiName
wmiEventNamespace
restartTure
dbName
clientPrg
cRealm
queryResults
queryStatus
targetObject
responseMsg
responseHeader
customerId
loginUser
settings
attackSpeed
attackSignature
creationUtcTime
targetFilename
grantedAccess
destProcessGUID
signatureStatus
responseCode
accessAgent
responseTime
maxTTLtoClient
attributeRegion
attributeCompanyPath
attributeCompanyName
signature
responseAddress
minTTLtoClient
maxTTLtoServer
signed
requestUrl
requestTime
requestHeader
rawPayload
gid
fileVersion
requestContext
cmdContent
requestUrlQuery
confidence
pcapRecord
sourceEventIds
destIsIpv6
attributeCopanyId
requestClientApplication
sandboxReportId
ruleLevel
requestApplicationMsg
srcIsIpv6
requestBody
rawEvent
imageLoaded
rdpServerSupports
HLShift
registrationCodeNodeId
initiated
parentCommandLine
productVendorName
pktsOut
srcOrgId
parentImage
parentProcessId
fromCustomStrategy
pktsIn
payload
originator
tacticId
scanPortList
nxAlertStatus
parentProcessGuid
integrityLevel
terminalSessionId
filePath
oldFileType
oldFileSize
scanIpList
nxAlertLevel
scanArpMacList
organizationId
parentProcessMd5
oldFileList
oldFileHash
familyId
subTechniquesName
sidHistory
objectDN
logonGuid
currentDirectory
parentProcessUserName
parentProcessName
oldFileExtension
tags
recordNum
eventTime
createTime
execProcessId
commandLine
image
processId
name
minTTLtoServer
nfsStatus
transMode
routerName
unixTty
tlsIssuer
scriptId
scriptContent
unixOpResults
toAlarm
AiLPHAPartID
processChain
processMd5
processUserName
message
subTechniquesId
samAccountName
serverPrincipalName
mailSubject
messageTotal
processName
evidence
IoC
tmpList
mailType
mailTitle
accessList
ailphaIDSAlertLog
unixPath
logname
requestOS
srcGeoLatitude
srcGeoIsp
loginOutTrue
smbClientVersion
smbServerVersion
attackerUnitPath
deviceHostAssetId
ailphaWebAlertLog
transactionIdentifier
unixOpSuccess
unixExitValue
destHostAssetId
fileArchived
confidenceLevel
virusFamily
srcProcessGUID
driveDevice
fileMd5
vlanId
fileList
fileHash
locality
interfaceName
interfaceId
externalId
confidenceScore
credibility
messageNumber
shareLocalPath
unixEuidName
instanceName
icmpBytes
httpVersion
srcGeoPostalCode
srcGeoLongitude
accountLocked
accessMask
pnpDeviceName
tokenElevationType
httpRefererDnsDomain
fileType
cNameType
clientConnectionHint
timesFrom
mailHead
modelType
modelName
unixGidName
unixFsuidName
unixFsgidName
destUserClass
traceId
clipboardClientInfo
sha256
deployment
operation
alertType
httpReferer
hostName
destGroupName
destGeoRegionCode
threatSeverity
fileSize
requestDomain
instanceId
srcThreadId
unixAuidName
unixUidName
tag
dhcpMsgType
packetIdentifier
cNames
privilegeList
pipeName
srcUserGroupName
srcUserGroupId
vlanName
pktsInAndOut
appProtocol
destUserAccount
srcUserClass
srcUserAccount
IoCAttackerPortrait
netId
serviceStartType
serviceBinPath
taskContent
killChain
chineseModelName
unixSuidName
nfsServerVersion
sNameType
requestContentType
describe
srcServiceName
oldFileName
victimSecurityZoneName
attackerSecurityZoneName
destSecurityZoneName
srcSecurityZoneName
dascaDeviceId
serviceName
fileName
attackStrategy
threatName
malwareRecord
TIType
srcUnitName
srcUnitRegion
dispatchSource
unixUserShell
srcUnitIndustry
srcUnitPath
srcSecurityTag
destUnitId
destUnitName
taskName
destUnitRegion
unixSgidName
unixEgidName
destUnitPath
destSecurityTag
attackerUnitId
attackerUnitRegion
attackerUnitIndustry
attackerSecurityTag
victimUnitId
victimUnitName
victimUnitRegion
victimUnitIndustry
victimUnitPath
victimSecurityTag
unixUid
icmpAnswerCode
icmpAnswerType
icmpQueryCode
icmpQueryType
mqttMessage
mqttVersion
mqttControlPacketFlag
mqttControlPacketType
functionCode
unitIdentifier
protocolIdentifier
requestControlMsg
eventCount
errorMessage
attackIntent
clientToolName
tlsSni
responseOS
processGuid
utcTime
victim
srcZone
lmPackageName
ruleType
unixType
queryName
sysmonConfigurationFileHash
sysmonConfigurationFile
sysmonSchemaVersion
sysmonVersion
attributeCompanyId
sysmonState
previousCreationUtcTime
isAPT
machineCode
IoCType
TIName
IoCLevel
IoCThreatName
IoCHash
rawKillChain
attackerAddress
victimAddress
timePasswordLastSet
aiLoggerConfigurationFileHash
errorCode
endTime
logTypeId
effectRow
aiLoggerSchemaVersion
aiLoggerConfigurationFile
aiLoggerState
destImage
aiLoggerVersion
sourceImage
fileIsExecutable
newThreadId
elevatedToken
subStatus
callTrace
regNewName
fileContents
PcapFileName
sRealm
rdpChannels
tls_x509Serials
saslMechanism
nfsType
ldapSearchScope
DN
clientCapabilities
ldapVersion
attackerUnitName
clientKBLayout
objectType
tlsSerial
requestParameters
requestMethod
deviceVendor
alarmStatus
clientOperatingSystem
dvcOutInterface
dvcInInterface
dvcDomain
failReason
destLoginId
srcLoginId
description
logVersion
sclass
tacticName
suggestion
victimSecurityZone
XFF
threadId
opType
tcpFlagsTs
tcpFlagsTc
subCategory
category
alarmTag
flowSource
tcpFlags
tcpSyn
tcpState
dvcDirection
passwd
DPIAlertType
destSecurityZone
sNames
tcpAck
responseApplicationMsg
responseControlMsg
flowState
flowReason
dnsType
sshServerSWInfo
sshServerVersion
sshClientSWInfo
sshClientVersion
tlsFIngerPrint
tlsIssuerDn
requestLM
cve
srcUserId
cmdContentData
ntlmsspDomain
responseContentType
dvcAction
srcSecurityZone
direction
deviceAssetType
deviceAssetSubType
alarmType
dbappWafAlertLog
mailFromDomain
protocolType
queryType
webAccess
vulnerability
virusVesion
mclass
flowVersion
virusType
virusName
udpBytes
transProtocol
suffix
startTime
srcVlanName
tcpRst
clientKBFunctionKeys
srcVlanId
srcUserPrivileges
srcUserName
srcTransZone
srcTransPort
srcTransAddress
mailContent
srcProcessName
srcProcessId
srcPort
durationTime
clientProductId
clientProductIdString
srcNtDomain
srcMacAddress
srcHostName
srcGeoRegionCode
srcGeoRegion
loginType
loginId
destUserGroupName
destUserGroupId
deviceSendProductName
deviceProtocol
deviceName
deviceModel
deviceId
deviceHostname
deviceCat
deviceAssetTypeId
deviceAssetSubTypeId
deviceAddress
destZone
destVlanName
destVlanId
destUserPrivileges
status
shareName
destUserName
destUserId
alarmName
techniquesId
techniquesName
origin
bytesOut
bytesInAndOut
srcAddress
destTransZone
destTransPort
destTransAddress
destServiceName
destProcessName
destProcessId
destPort
destNtDomain
destHostName
destGeoRegion
dirName
destGeoPostalCode
destGeoLatitude
ailphaVerify
destGeoIsp
destGeoId
destGeoCounty
destGeoCountryCode
destGeoCountry
rdpClientVersion
clientPrincipalName
clientKBType
ldapOpType
alarmSource
attributeIndustry
groupByFields
destOrgId
srcHostAssetId
requestCookies
sql
srcTransMacAddress
flowProtocol
flowProtocl
srcGeoId
bruteSrcIPCout
rdpProtocol
authPackageName
startAddress
tlsSubject
responseLM
startFunction
attackSource
attackTarget
affectProduct
logSessionId
oldFilePath
eventNum
attackMethod
dataType
attacker
destGeoLongitude
eventId
catObject
startModule
alarmResults
alarmDescription
sendHostAddress
deviceReceiptTime
deviceProductType
destMacAddress
eventType
logonId
regValue
TIMatchField
hostAddress
serviceAccount
rpcAuthType
keyLength
logonProcessName
destUnitIndustry
srcUnitId
clipboardSession
unixAcct
flowDestPort
bindDN
ticketHash
srcChannel
dataSource
eventIDs
destAddress
dhcpRenewalTime
serviceType
bccUserName
tlsVersion