use super::Reporter;

impl Reporter {
    pub(super) fn diagnose_personality(&self, ptype: &str, score: f64) -> String {
        let severity = if score >= 80.0 {
            if self.is_zh() {
                "严重"
            } else {
                "severe"
            }
        } else if score >= 60.0 {
            if self.is_zh() {
                "中度"
            } else {
                "moderate"
            }
        } else {
            if self.is_zh() {
                "轻度"
            } else {
                "mild"
            }
        };
        let cause = match ptype {
            "The Copy-Paste Artist" => {
                if self.is_zh() {
                    "重复代码驱动开发综合症"
                } else {
                    "duplication-driven development syndrome"
                }
            }
            "The YOLO Engineer" => {
                if self.is_zh() {
                    "恐慌驱动开发综合症"
                } else {
                    "panic-driven development syndrome"
                }
            }
            "The Trait Wizard" => {
                if self.is_zh() {
                    "复杂度膨胀综合症"
                } else {
                    "complexity inflation syndrome"
                }
            }
            "The Legacy Necromancer" => {
                if self.is_zh() {
                    "语义退化综合症"
                } else {
                    "semantic decay syndrome"
                }
            }
            "The Hotfix Mercenary" => {
                if self.is_zh() {
                    "技术债累积综合症"
                } else {
                    "technical debt accumulation syndrome"
                }
            }
            "The Startup Survivor" => {
                if self.is_zh() {
                    "增长代价综合症"
                } else {
                    "growth-at-all-costs syndrome"
                }
            }
            "The Academic Wizard" => {
                if self.is_zh() {
                    "抽象过度综合症"
                } else {
                    "abstraction overdose syndrome"
                }
            }
            _ => {
                if self.is_zh() {
                    "代码异味综合症"
                } else {
                    "code smell syndrome"
                }
            }
        };
        format!("{severity} {cause}")
    }

    #[expect(dead_code)]
    pub(super) fn rule_display_name(&self, rule_name: &str) -> String {
        let name = match (self.i18n.lang.as_str(), rule_name) {
            ("zh-CN", r) => match r {
                "panic-abuse" => "panic滥用",
                "god-function" => "上帝函数",
                "magic-number" => "魔法数字",
                "todo-comment" => "TODO注释",
                "println-debugging" => "println调试",
                "code-duplication" => "代码重复",
                "cross-file-duplication" => "跨文件重复",
                "deep-nesting" => "深层嵌套",
                "long-function" => "过长函数",
                "file-too-long" => "过长文件",
                "single-letter-variable" => "单字母变量",
                "terrible-naming" => "糟糕命名",
                "meaningless-naming" => "无意义命名",
                "hungarian-notation" => "匈牙利命名",
                "abbreviation-abuse" => "过度缩写",
                "unwrap-abuse" => "unwrap滥用",
                "box-abuse" => "Box滥用",
                "duplicate-imports" => "重复导入",
                "bare-except" => "裸except",
                "any-type" => "any类型",
                _ => return rule_name.replace('-', " "),
            },
            (_, name) => return name.replace('-', " "),
        };
        name.to_string()
    }

    pub(super) fn translate_personality_type<'a>(&self, en: &'a str) -> &'a str {
        if !self.is_zh() {
            return en;
        }
        match en {
            "The Copy-Paste Artist" => "复制粘贴艺术家",
            "The YOLO Engineer" => "YOLO工程师",
            "The Trait Wizard" => "特质巫师",
            "The Legacy Necromancer" => "遗留代码亡灵法师",
            "The Hotfix Mercenary" => "热修复雇佣兵",
            "The Startup Survivor" => "创业幸存者",
            "The Academic Wizard" => "学术巫师",
            "The Enterprise Bureaucrat" => "企业官僚",
            _ => en,
        }
    }

    pub(super) fn translate_threat<'a>(&self, en: &'a str) -> &'a str {
        if !self.is_zh() {
            return en;
        }
        match en {
            "☢ CRITICAL" => "☢ 危险",
            "⚠ HIGH" => "⚠ 高危",
            "⚠ ELEVATED" => "⚠ 升高",
            "🟢 MODERATE" => "🟢 中等",
            _ => en,
        }
    }

    pub(super) fn translate_trait<'a>(&self, en: &'a str) -> &'a str {
        if !self.is_zh() {
            return en;
        }
        match en {
            "Ctrl+C, Ctrl+V is your IDE's most used shortcut" => {
                "Ctrl+C、Ctrl+V是你IDE最常用的快捷键"
            }
            "Why abstract when you can duplicate" => "能复制何必抽象",
            "Same bug in 5 places = 5x debugging fun" => "同一个bug出现在5个地方 = 5倍调试乐趣",
            "DRY stands for 'Don't Repeat... wait, too late'" => "DRY代表'不要重复……等等，太晚了'",
            "Believes every Result is Ok" => "坚信每个Result都是Ok",
            "unwrap() used more often than error handling" => "unwrap()比错误处理用得还多",
            "Production incidents are just 'surprise features'" => "生产事故不过是'惊喜功能'",
            "Has never met a None they couldn't ignore" => "从未遇到过不能忽略的None",
            "Loves building pyramids of doom" => "热爱建造末日金字塔",
            "Each function is a journey through layers of abstraction" => {
                "每个函数都是穿越抽象层的旅程"
            }
            "Indentation is a competitive sport" => "缩进是一项竞技运动",
            "Thinks 'extract method' is for amateurs" => "认为'提取方法'是业余爱好者的做法",
            "Why use many word when few letter do trick" => "能用短名何必用长名",
            "Variables named like chess coordinates" => "变量命名像棋盘坐标",
            "'data', 'temp', 'val' — the holy trinity of naming" => {
                "'data'、'temp'、'val' — 命名三圣"
            }
            "Has never heard of domain-driven design" => "从未听说过领域驱动设计",
            "More TODOs than actual features" => "TODO比实际功能还多",
            "'TODO: fix this later' — later never came" => "'TODO: 以后修复' — 以后从未到来",
            "Commits with 'temp', 'test', 'asdf' messages" => "提交信息是'temp'、'test'、'asdf'",
            "WIP is a lifestyle, not a branch name" => "WIP是生活方式，不是分支名",
            "Shipped fast, now paying the interest" => "快速上线，现在在还债",
            "Copy-paste driven development" => "复制粘贴驱动开发",
            "Production incidents are 'learning experiences'" => "生产事故是'学习机会'",
            "Technical debt is just 'future velocity'" => "技术债只是'未来速度'",
            "Loves building pyramids of abstraction" => "热爱建造抽象金字塔",
            "Each function is a thesis in disguise" => "每个函数都是伪装的论文",
            "Naming conventions from another dimension" => "来自异次元的命名规范",
            "If it's not complex, it's not 'elegant'" => "不够复杂就不够'优雅'",
            "A balanced mix of code smells" => "代码异味的平衡组合",
            "Not great at anything, not terrible at anything" => "什么都不精通，什么都不算差",
            "Jack of all trades, master of technical debt" => "样样通，技术债专精",
            _ => en,
        }
    }

    pub(super) fn translate_emotion<'a>(&self, en: &'a str) -> &'a str {
        if !self.is_zh() {
            return en;
        }
        match en {
            "numb from repetitive work" => "重复劳动导致麻木",
            "denial" => "否认",
            "proud of the complexity" => "为复杂度自豪",
            "confident in their abbreviations" => "对自己的缩写充满信心",
            "overwhelmed but optimistic" => "不堪重负但保持乐观",
            "battle-scarred but still shipping" => "伤痕累累但仍在交付",
            "intellectually satisfied, practically lost" => "理论上满足，实践中迷失",
            "professional detachment" => "职业性超脱",
            _ => en,
        }
    }

    pub(super) fn translate_philosophy<'a>(&self, en: &'a str) -> &'a str {
        if !self.is_zh() {
            return en;
        }
        match en {
            "it worked once, it'll work again" => "成功过一次，就会再成功",
            "it won't crash in production" => "生产环境不会崩溃的",
            "if it's not nested, it's not sophisticated" => "不够嵌套就不够精妙",
            "comments are for people who can't read code" => "注释是给读不懂代码的人看的",
            "future me will deal with it" => "未来的我会处理的",
            "move fast and definitely break things" => "快速行动，必然出错",
            "the theory is beautiful, the practice is secondary" => "理论很美，实践其次",
            "it compiles, it ships" => "能编译就能上线",
            _ => en,
        }
    }

    pub(super) fn translate_cause_of_death<'a>(&self, en: &'a str) -> &'a str {
        if !self.is_zh() {
            return en;
        }
        match en {
            "uncontrolled duplication metastasis" => "失控的重复代码转移",
            "panic-driven development" => "恐慌驱动开发",
            "complexity collapse — nesting level exceeded event horizon" => {
                "复杂度坍塌 — 嵌套层级超过事件视界"
            }
            "semantic starvation — variable names lost all meaning" => {
                "语义饥饿 — 变量名完全丧失含义"
            }
            "TODO accumulation — promises exceeded delivery capacity" => {
                "TODO堆积 — 承诺超出交付能力"
            }
            "growth-at-all-costs syndrome" => "不惜一切代价增长综合症",
            "abstraction overdose — complexity exceeded comprehension" => {
                "抽象过度 — 复杂度超出理解范围"
            }
            "death by a thousand paper cuts" => "千刀万剐",
            _ => en,
        }
    }

    pub(super) fn translate_condition<'a>(&self, en: &'a str) -> &'a str {
        if !self.is_zh() {
            return en;
        }
        match en {
            "terminal — palliative care recommended" => "晚期 — 建议姑息治疗",
            "critical but treatable" => "危重但可治疗",
            "critical — needs immediate Result therapy" => "危重 — 需要立即Result治疗",
            "stable but concerning" => "稳定但令人担忧",
            "severe — god functions threatening readability" => "严重 — 上帝函数威胁可读性",
            "moderate — some functions need splitting" => "中度 — 部分函数需要拆分",
            "chronic but manageable" => "慢性但可控",
            "stable with high technical debt interest" => "稳定但技术债利息很高",
            "critical — needs stabilization sprint" => "危重 — 需要稳定化冲刺",
            "chronic — needs simplification therapy" => "慢性 — 需要简化治疗",
            "fair — routine maintenance recommended" => "尚可 — 建议常规维护",
            _ => en,
        }
    }

    pub(super) fn translate_final_words<'a>(&self, en: &'a str) -> &'a str {
        if !self.is_zh() {
            return en;
        }
        match en {
            "just one more hotfix" => "再修一个热修复就好",
            "i'll add error handling later" => "以后再加错误处理",
            "i can still understand it" => "我还是能看懂的",
            "temp2 should work" => "temp2应该能用",
            "i'll fix it in the next sprint" => "下个冲刺再修",
            "we'll fix it after the next funding round" => "下一轮融资后就修",
            "it's actually quite simple once you understand the theory" => "理解了理论其实很简单",
            "we'll refactor next quarter" => "下个季度重构",
            _ => en,
        }
    }
}
