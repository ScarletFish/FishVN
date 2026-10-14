# FDD 规范摘要（Feature-Driven Development）

来源：Jeff De Luca 与 Peter Coad 于 1997 年新加坡银行项目提出，成文于《Java Modeling in Color with UML》(1999) 与《A Practical Guide to Feature-Driven Development》(2002)。

## 五个过程

1. Develop an Overall Model —— 开发整体模型
2. Build a Feature List —— 构建功能列表
3. Plan by Feature —— 按功能计划
4. Design by Feature —— 按功能设计
5. Build by Feature —— 按功能构建

前两步在项目初期做一次，后三步对每个功能集迭代进行。

## 功能（Feature）定义

- 一个小的、对客户有价值的功能。
- 可由单人两周内完成；超过两周需拆小。
- 命名模板：`<动作> <结果> <对象>`，连接词（of / for / to / from / by）可选。
  - 例：Calculate the total of a sale / Validate the password of a user / Generate the monthly statement for a customer

## 功能列表的三层层级

- **主题域 Subject Area**：最大的业务功能分组，常命名为「XX 管理」。
- **活动集 Feature Set / Activity Set**：一个业务活动（如「管理一部作品」）。
- **功能 Feature**：具体功能。

## 整体模型 = 领域模型

- 用业务概念描述系统中的关键抽象及其关系，
- 由领域专家与架构师共建，目的是达成共享理解，不是产出完整规格。

## 最佳实践（节选）

领域对象建模 / 按功能开发 / 类（代码）个人所有制 / 功能小组 / 审查 / 定期构建 / 配置管理 / 进度可见性。
