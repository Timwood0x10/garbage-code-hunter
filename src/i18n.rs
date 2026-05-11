use std::collections::HashMap;

pub struct I18n {
    pub lang: String,
    messages: HashMap<String, HashMap<String, String>>,
}

impl I18n {
    pub fn new(lang: &str) -> Self {
        // Normalize language code
        let normalized_lang = match lang.to_lowercase().replace("_", "-").as_str() {
            "en" | "en-us" | "english" => "en-US",
            "zh" | "zh-cn" | "chinese" => "zh-CN",
            _ => lang,
        };

        let mut messages = HashMap::new();

        // chinese messages
        let mut zh_cn = HashMap::new();
        zh_cn.insert("title".to_string(), "🗑️  垃圾代码猎人 🗑️".to_string());
        zh_cn.insert(
            "preparing".to_string(),
            "正在准备吐槽你的代码...".to_string(),
        );
        zh_cn.insert(
            "report_title".to_string(),
            "📊 垃圾代码检测报告".to_string(),
        );
        zh_cn.insert(
            "found_issues".to_string(),
            "发现了一些需要改进的地方：".to_string(),
        );
        zh_cn.insert("statistics".to_string(), "📈 问题统计:".to_string());
        zh_cn.insert(
            "nuclear_issues".to_string(),
            "🔥 核弹级问题 (需要立即修复)".to_string(),
        );
        zh_cn.insert(
            "spicy_issues".to_string(),
            "🌶️  辣眼睛问题 (建议修复)".to_string(),
        );
        zh_cn.insert(
            "mild_issues".to_string(),
            "😐 轻微问题 (可以忽略)".to_string(),
        );
        zh_cn.insert("total".to_string(), "📝 总计".to_string());
        zh_cn.insert("summary".to_string(), "📋 总结".to_string());
        zh_cn.insert("suggestions".to_string(), "💡 改进建议".to_string());
        zh_cn.insert(
            "clean_code".to_string(),
            "🎉 哇！你的代码居然没有明显的垃圾！".to_string(),
        );
        zh_cn.insert(
            "clean_code_warning".to_string(),
            "但是别高兴太早，也许你的逻辑有问题我还没检测到 😏".to_string(),
        );
        zh_cn.insert(
            "keep_improving".to_string(),
            "继续努力，让代码变得更好！🚀".to_string(),
        );
        zh_cn.insert("top_files".to_string(), "🏆 问题最多的文件".to_string());
        zh_cn.insert("detailed_analysis".to_string(), "🔍 详细分析".to_string());

        // english messages
        let mut en_us = HashMap::new();
        en_us.insert(
            "title".to_string(),
            "🗑️  Garbage Code Hunter 🗑️".to_string(),
        );
        en_us.insert(
            "preparing".to_string(),
            "Preparing to roast your code...".to_string(),
        );
        en_us.insert(
            "report_title".to_string(),
            "📊 Code Quality Report".to_string(),
        );
        en_us.insert(
            "found_issues".to_string(),
            "Found some areas for improvement:".to_string(),
        );
        en_us.insert("statistics".to_string(), "📈 Issue Statistics:".to_string());
        en_us.insert(
            "nuclear_issues".to_string(),
            "🔥 Nuclear Issues (fix immediately)".to_string(),
        );
        en_us.insert(
            "spicy_issues".to_string(),
            "🌶️  Spicy Issues (should fix)".to_string(),
        );
        en_us.insert(
            "mild_issues".to_string(),
            "😐 Mild Issues (can ignore)".to_string(),
        );
        en_us.insert("total".to_string(), "📝 Total".to_string());
        en_us.insert("summary".to_string(), "📋 Summary".to_string());
        en_us.insert("suggestions".to_string(), "💡 Suggestions".to_string());
        en_us.insert(
            "clean_code".to_string(),
            "🎉 Wow! Your code doesn't have obvious garbage!".to_string(),
        );
        en_us.insert(
            "clean_code_warning".to_string(),
            "But don't celebrate too early, maybe there are logic issues I haven't detected 😏"
                .to_string(),
        );
        en_us.insert(
            "keep_improving".to_string(),
            "Keep working hard to make your code better! 🚀".to_string(),
        );
        en_us.insert(
            "top_files".to_string(),
            "🏆 Files with Most Issues".to_string(),
        );
        en_us.insert(
            "detailed_analysis".to_string(),
            "🔍 Detailed Analysis".to_string(),
        );

        messages.insert("zh-CN".to_string(), zh_cn);
        messages.insert("en-US".to_string(), en_us);

        Self {
            lang: normalized_lang.to_string(),
            messages,
        }
    }

    pub fn get(&self, key: &str) -> String {
        self.messages
            .get(&self.lang)
            .and_then(|lang_map| lang_map.get(key))
            .cloned()
            .unwrap_or_else(|| {
                // Fallback to English
                self.messages
                    .get("en-US")
                    .and_then(|lang_map| lang_map.get(key))
                    .cloned()
                    .unwrap_or_else(|| format!("Missing translation: {key}"))
            })
    }

    pub fn get_roast_messages(&self, rule_name: &str) -> Vec<String> {
        match (self.lang.as_str(), rule_name) {
            ("zh-CN", "terrible-naming") => vec![
                "这个变量名比我的编程技能还要抽象，而我连 Hello World 都写不对".to_string(),
                "这个名字告诉我你已经放弃治疗了，建议直接转行卖煎饼果子".to_string(),
                "用这个做变量名？你是想让维护代码的人哭着辞职吗？".to_string(),
                "恭喜你发明了最没有意义的变量名，可以申请专利了".to_string(),
                "这变量名就像'无名氏'一样毫无特色，连我奶奶都能起个更好的名字".to_string(),
                "看到这个变量名，我的智商都下降了，现在只能数到3了".to_string(),
                "这变量名的创意程度约等于给孩子起名叫'小明'".to_string(),
                "你这变量名让我想起了我的前任——毫无意义且令人困惑".to_string(),
                "这个变量名的描述性约等于'东西'这个词的精确度".to_string(),
                "恭喜！你成功地让变量名比注释还要难懂".to_string(),
            ],
            ("zh-CN", "single-letter-variable") => vec![
                "单字母变量？你是在写数学公式还是在考验我的猜谜能力？".to_string(),
                "这是变量名还是你键盘只剩一个键能用了？".to_string(),
                "用单字母做变量名，建议你去买本《变量命名从入门到放弃》".to_string(),
                "单字母变量：让代码比古埃及象形文字还难懂的神器".to_string(),
                "这个变量名短得像我对你代码技能的期望一样".to_string(),
                "单字母变量名？你这是在玩猜字谜游戏吗？".to_string(),
                "恭喜你用一个字母成功地表达了无穷的困惑".to_string(),
                "这变量名的信息量约等于一个句号".to_string(),
                "你这是在节约字符还是在为难后来的维护者？".to_string(),
            ],
            ("zh-CN", "deep-nesting") => vec![
                "这嵌套层数比俄罗斯套娃还要深，你是在挑战人类的理解极限吗？".to_string(),
                "嵌套这么深，是想挖到地心还是想让读代码的人迷路？".to_string(),
                "这代码嵌套得像洋葱一样，剥一层哭一次".to_string(),
                "嵌套层数超标！建议重构，或者直接删了重写".to_string(),
                "这嵌套深度已经可以申请吉尼斯世界纪录了，类别：最令人绝望的代码".to_string(),
                "这代码比盗梦空间还要复杂，至少电影还有字幕".to_string(),
                "你这嵌套层数让我想起了我的人际关系——复杂且令人困惑".to_string(),
                "这嵌套深度足够埋葬我对编程的热情了".to_string(),
                "恭喜你成功地把简单问题复杂化，这是一门艺术".to_string(),
                "这代码的嵌套层数比我的焦虑层数还要多".to_string(),
            ],
            ("zh-CN", "long-function") => vec![
                "这个函数比我的简历还要长，而我的简历已经长到HR看了想哭！".to_string(),
                "函数长度建议拆分成几个小函数，或者直接删了重新做人".to_string(),
                "这么长的函数？你是想让人一口气读完然后当场去世吗？".to_string(),
                "这个函数比我对前任的怨念还要长".to_string(),
                "这函数长得像《战争与和平》一样，但至少托尔斯泰会分章节".to_string(),
                "这个函数需要一个GPS才能导航到结尾".to_string(),
                "你这函数长度已经超越了我的注意力极限".to_string(),
                "这函数比我妈的唠叨还要长，至少我妈会换个话题".to_string(),
                "恭喜你写出了一个需要分期付款才能读完的函数".to_string(),
                "这函数的长度让我怀疑你是按行数计工资的".to_string(),
            ],
            ("zh-CN", "unwrap-abuse") => vec![
                "又一个 unwrap()！你是想让程序在生产环境里爆炸给老板看烟花吗？".to_string(),
                "unwrap() 大师！错误处理是什么？能吃吗？还是说你觉得错误不会发生？".to_string(),
                "看到这个 unwrap()，我仿佛听到了运维工程师的哭声".to_string(),
                "unwrap() 使用者，恭喜你获得了'半夜被电话吵醒专业户'称号".to_string(),
                "这个 unwrap() 就像俄罗斯轮盘赌，总有一发是实弹".to_string(),
                "unwrap()：让程序员体验心脏病发作的最佳工具".to_string(),
                "你这 unwrap() 用得比我吃泡面还频繁，至少泡面不会让程序崩溃".to_string(),
                "unwrap() 狂魔！你是不是觉得 panic 很好玩？".to_string(),
                "这么多 unwrap()，你确定不是在写自毁程序？".to_string(),
                "unwrap() 使用过度，建议改名为 'panic_generator.rs'".to_string(),
            ],
            ("zh-CN", "unnecessary-clone") => vec![
                "clone() 狂魔！你是想把内存用完还是想让电脑罢工？".to_string(),
                "这么多 clone()，你确定不是从 Java 转过来的？".to_string(),
                "clone() 使用过度！Rust 的借用检查器已经哭晕在厕所".to_string(),
                "又见 clone()！建议你重新学习 Rust，或者改学 Python".to_string(),
                "这些 clone() 让我想起了复印机店的老板——疯狂复制一切".to_string(),
                "clone() 滥用：让内存管理专家失业的最佳方式".to_string(),
                "你这 clone() 用得比我换袜子还频繁".to_string(),
                "恭喜你成功地把零拷贝变成了无限拷贝".to_string(),
                "这么多 clone()，你是不是觉得内存是免费的？".to_string(),
                "clone() 大师！你已经掌握了如何让程序跑得像蜗牛一样慢".to_string(),
            ],
            ("zh-CN", "complex-closure") => vec![
                "闭包套闭包？你这是在写俄罗斯套娃还是在考验读者的智商？".to_string(),
                "嵌套闭包比我的人际关系还复杂".to_string(),
                "这闭包嵌套得像洋葱一样，剥一层哭一次".to_string(),
                "闭包嵌套过深，建议拆分成独立函数".to_string(),
                "这个闭包的参数比我的借口还多".to_string(),
                "闭包参数过多，你确定不是在写函数？".to_string(),
            ],
            ("zh-CN", "lifetime-abuse") => vec![
                "生命周期标注比我的生命还复杂".to_string(),
                "这么多生命周期，你是在写哲学论文吗？".to_string(),
                "生命周期滥用，建议重新设计数据结构".to_string(),
                "生命周期多到让人怀疑人生".to_string(),
            ],
            ("zh-CN", "trait-complexity") => vec![
                "这个 trait 的方法比我的借口还多".to_string(),
                "trait 方法过多，违反了单一职责原则".to_string(),
                "这个 trait 比瑞士军刀还要全能".to_string(),
                "trait 臃肿，建议拆分成多个小 trait".to_string(),
                "泛型参数比我的密码还复杂".to_string(),
                "这么多泛型，你是在写数学公式吗？".to_string(),
            ],
            ("zh-CN", "generic-abuse") => vec![
                "泛型参数比我的购物清单还长".to_string(),
                "这么多泛型，编译器都要哭了".to_string(),
                "泛型滥用，建议重新设计架构".to_string(),
                "泛型多到让人怀疑这还是 Rust 吗".to_string(),
                "泛型参数的命名创意约等于零".to_string(),
                "泛型名字比我的耐心还短".to_string(),
            ],
            ("zh-CN", "channel-abuse") => vec![
                "Channel 用得比我发微信还频繁，你确定不是在写聊天软件？".to_string(),
                "这么多 Channel，你是想开通讯公司吗？".to_string(),
                "Channel 滥用！你的程序比电话交换机还复杂".to_string(),
                "Channel 数量超标，建议重新设计架构".to_string(),
                "这么多 Channel，我怀疑你在写分布式系统".to_string(),
            ],
            ("zh-CN", "async-abuse") => vec![
                "Async 函数比我的异步人生还要复杂".to_string(),
                "这么多 async，你确定不是在写 JavaScript？".to_string(),
                "Async 滥用！建议学习一下同步编程的美好".to_string(),
                "异步函数过多，小心把自己绕晕了".to_string(),
                "Await 用得比我等外卖还频繁".to_string(),
                "这么多 await，你的程序是在等什么？世界末日吗？".to_string(),
            ],
            ("zh-CN", "dyn-trait-abuse") => vec![
                "Dyn trait 用得比我换工作还频繁".to_string(),
                "这么多动态分发，性能都跑到哪里去了？".to_string(),
                "Dyn trait 滥用，你确定不是在写 Python？".to_string(),
                "动态 trait 过多，编译器优化都哭了".to_string(),
                "这么多 dyn，你的程序比变色龙还善变".to_string(),
            ],
            ("zh-CN", "unsafe-abuse") => vec![
                "Unsafe 代码！你这是在玩火还是在挑战 Rust 的底线？".to_string(),
                "又见 unsafe！安全性是什么？能吃吗？".to_string(),
                "Unsafe 使用者，恭喜你获得了'内存安全破坏者'称号".to_string(),
                "这个 unsafe 让我想起了 C 语言的恐怖回忆".to_string(),
                "Unsafe 代码：让 Rust 程序员夜不能寐的存在".to_string(),
            ],
            ("zh-CN", "ffi-abuse") => vec![
                "FFI 滥用！你这是在和多少种语言谈恋爱？".to_string(),
                "外部接口比我的社交关系还复杂！".to_string(),
                "这么多 FFI，Rust 的安全性都要哭了".to_string(),
                "C 语言接口过多，你确定这还是 Rust 项目？".to_string(),
                "FFI 代码让我想起了指针地狱的恐怖".to_string(),
            ],
            ("zh-CN", "macro-abuse") => vec![
                "宏定义比我的借口还多".to_string(),
                "这么多宏，你确定不是在写 C 语言？".to_string(),
                "宏滥用！编译时间都被你搞长了".to_string(),
                "宏过多，调试的时候准备哭吧".to_string(),
                "这么多宏，IDE 都要罢工了".to_string(),
            ],
            ("zh-CN", "module-complexity") => vec![
                "模块嵌套比俄罗斯套娃还深".to_string(),
                "这模块结构比我的家族关系还复杂".to_string(),
                "模块嵌套过深，建议重新组织代码结构".to_string(),
                "这么深的模块，找个函数比找宝藏还难".to_string(),
            ],
            ("zh-CN", "pattern-matching-abuse") => vec![
                "模式匹配比我的感情生活还复杂".to_string(),
                "这么多模式，你是在写解谜游戏吗？".to_string(),
                "模式过多，建议简化逻辑".to_string(),
                "复杂的模式让代码可读性直线下降".to_string(),
                "Match 分支比我的人生选择还多".to_string(),
                "这么多 match 分支，你确定不是在写状态机？".to_string(),
            ],
            ("zh-CN", "reference-abuse") => vec![
                "引用比我的社交关系还复杂".to_string(),
                "这么多引用，你确定不是在写指针迷宫？".to_string(),
                "引用过多，小心借用检查器罢工".to_string(),
                "引用数量超标，建议重新设计数据结构".to_string(),
            ],
            ("zh-CN", "box-abuse") => vec![
                "Box 用得比快递还频繁".to_string(),
                "这么多 Box，你是在开仓库吗？".to_string(),
                "Box 过多，堆内存都要爆炸了".to_string(),
                "Box 滥用，建议考虑栈分配".to_string(),
                "这么多 Box，内存分配器都累了".to_string(),
            ],
            ("zh-CN", "slice-abuse") => vec![
                "切片比我切菜还频繁".to_string(),
                "这么多切片，你是在开水果店吗？".to_string(),
                "切片过多，数组都被你切碎了".to_string(),
                "Slice 滥用，建议使用 Vec".to_string(),
            ],
            ("zh-CN", "code-duplication") => vec![
                "检测到重复代码！你是复制粘贴大师吗？".to_string(),
                "这些重复代码比双胞胎还像".to_string(),
                "DRY原则哭了，你的代码湿得像雨季".to_string(),
                "重复代码这么多，建议改名为copy-paste.rs".to_string(),
            ],
            ("zh-CN", "cyclomatic-complexity") => vec![
                "圈复杂度爆表！这代码比迷宫还复杂".to_string(),
                "复杂度这么高，连AI都看不懂".to_string(),
                "这函数的复杂度已经超越人类理解范围".to_string(),
                "建议拆分函数，或者直接重写".to_string(),
            ],
            // English versions
            ("en-US", "terrible-naming") => vec![
                "This variable name is more abstract than my programming skills, and I can't even write Hello World correctly".to_string(),
                "This name tells me you've given up on life and should probably sell hotdogs instead".to_string(),
                "Using this as a variable name? Are you trying to make code maintainers cry and quit their jobs?".to_string(),
                "Congratulations on inventing the most meaningless variable name, you should patent this level of confusion".to_string(),
                "This variable name is as generic as 'John Doe', even my grandmother could come up with something better".to_string(),
                "Seeing this variable name, my IQ just dropped to single digits".to_string(),
                "This variable name has the creativity level of naming a kid 'Child'".to_string(),
                "Your variable name reminds me of my ex - meaningless and confusing".to_string(),
                "This variable name's descriptiveness is equivalent to calling everything 'stuff'".to_string(),
                "Congrats! You've successfully made variable names harder to understand than comments".to_string(),
            ],
            ("en-US", "single-letter-variable") => vec![
                "Single letter variable? Are you writing math formulas or testing my psychic abilities?".to_string(),
                "Is this a variable name or did your keyboard only have one working key?".to_string(),
                "Using single letters for variables, I suggest buying 'Variable Naming for Dummies'".to_string(),
                "Single letter variables: making code harder to read than ancient hieroglyphics".to_string(),
                "This variable name is as short as my expectations for your coding skills".to_string(),
                "Single letter variable name? Are we playing charades now?".to_string(),
                "Congrats on expressing infinite confusion with just one letter".to_string(),
                "This variable name has the information content of a period".to_string(),
                "Are you saving characters or just torturing future maintainers?".to_string(),
            ],
            ("en-US", "deep-nesting") => vec![
                "This nesting is deeper than Russian dolls, are you challenging the limits of human comprehension?".to_string(),
                "Nesting this deep, are you trying to dig to Earth's core or just make code readers get lost?".to_string(),
                "This code is nested like an onion, peel one layer, cry once".to_string(),
                "Nesting level exceeded! Suggest refactoring, or just delete and start over".to_string(),
                "This nesting depth could apply for a Guinness World Record in 'Most Despair-Inducing Code'".to_string(),
                "This code is more complex than Inception, at least the movie had subtitles".to_string(),
                "Your nesting levels remind me of my relationships - complex and confusing".to_string(),
                "This nesting depth is enough to bury my passion for programming".to_string(),
                "Congrats on successfully complicating simple problems, it's an art form".to_string(),
                "This code has more nesting levels than my anxiety layers".to_string(),
            ],
            ("en-US", "long-function") => vec![
                "This function is longer than my resume, and my resume already makes HR cry!".to_string(),
                "Function length suggests splitting into smaller functions, or just delete and start a new career".to_string(),
                "Such a long function? Are you trying to make people read it in one breath and die on the spot?".to_string(),
                "This function is longer than my grudges against my ex".to_string(),
                "This function is as long as 'War and Peace', but at least Tolstoy used chapters".to_string(),
                "This function needs a GPS to navigate to the end".to_string(),
                "Your function length has exceeded my attention span limits".to_string(),
                "This function is longer than my mom's nagging, at least she changes topics".to_string(),
                "Congrats on writing a function that requires installment payments to read completely".to_string(),
                "This function's length makes me suspect you're paid by lines of code".to_string(),
            ],
            ("en-US", "unwrap-abuse") => vec![
                "Another unwrap()! Are you trying to make the program explode in production like fireworks for your boss?".to_string(),
                "unwrap() master! What is error handling? Can you eat it? Or do you think errors just don't happen?".to_string(),
                "Seeing this unwrap(), I can almost hear the DevOps engineers crying".to_string(),
                "unwrap() user, congratulations on earning the 'Midnight Phone Call Specialist' title".to_string(),
                "This unwrap() is like Russian roulette, eventually you'll hit the real bullet".to_string(),
                "unwrap(): the best tool for experiencing heart attacks as a programmer".to_string(),
                "You use unwrap() more than I eat instant noodles, at least noodles don't crash programs".to_string(),
                "unwrap() maniac! Do you think panic is fun?".to_string(),
                "So many unwrap()s, are you sure you're not writing a self-destruct program?".to_string(),
                "unwrap() overuse detected, suggest renaming to 'panic_generator.rs'".to_string(),
            ],
            ("en-US", "unnecessary-clone") => vec![
                "clone() maniac! Are you trying to exhaust all memory or make the computer go on strike?".to_string(),
                "So many clone()s, are you sure you didn't just migrate from Java?".to_string(),
                "clone() overuse! Rust's borrow checker has fainted in the bathroom".to_string(),
                "Another clone()! Suggest relearning Rust, or maybe switch to Python".to_string(),
                "These clone()s remind me of a copy shop owner - frantically copying everything".to_string(),
                "clone() abuse: the best way to make memory management experts unemployed".to_string(),
                "You use clone() more frequently than I change socks".to_string(),
                "Congrats on successfully turning zero-copy into infinite-copy".to_string(),
                "So many clone()s, do you think memory is free?".to_string(),
                "clone() master! You've mastered how to make programs run as slow as snails".to_string(),
            ],
            ("en-US", "complex-closure") => vec![
                "Nested closures? Are you writing Russian dolls or testing readers' IQ?".to_string(),
                "Nested closures are more complex than my relationships".to_string(),
                "This closure nesting is like an onion, peel one layer, cry once".to_string(),
                "Closure nesting too deep, suggest splitting into separate functions".to_string(),
                "This closure has more parameters than my excuses".to_string(),
                "Too many closure parameters, are you sure you're not writing a function?".to_string(),
            ],
            ("en-US", "lifetime-abuse") => vec![
                "Lifetime annotations are more complex than my actual life".to_string(),
                "So many lifetimes, are you writing a philosophy paper?".to_string(),
                "Lifetime abuse, suggest redesigning data structures".to_string(),
                "So many lifetimes it makes me question existence".to_string(),
            ],
            ("en-US", "trait-complexity") => vec![
                "This trait has more methods than my excuses".to_string(),
                "Too many trait methods, violates single responsibility principle".to_string(),
                "This trait is more versatile than a Swiss Army knife".to_string(),
                "Bloated trait, suggest splitting into multiple smaller traits".to_string(),
                "Generic parameters more complex than my passwords".to_string(),
                "So many generics, are you writing mathematical formulas?".to_string(),
            ],
            ("en-US", "generic-abuse") => vec![
                "Generic parameters longer than my shopping list".to_string(),
                "So many generics, even the compiler is crying".to_string(),
                "Generic abuse, suggest redesigning architecture".to_string(),
                "So many generics, makes me wonder if this is still Rust".to_string(),
                "Generic parameter naming creativity equals zero".to_string(),
                "Generic name shorter than my patience".to_string(),
            ],
            ("en-US", "channel-abuse") => vec![
                "Using channels more frequently than I text, are you writing a chat app?".to_string(),
                "So many channels, are you starting a telecom company?".to_string(),
                "Channel abuse! Your program is more complex than a phone exchange".to_string(),
                "Channel count exceeded, suggest redesigning architecture".to_string(),
                "So many channels, I suspect you're writing a distributed system".to_string(),
            ],
            ("en-US", "async-abuse") => vec![
                "Async functions more complex than my asynchronous life".to_string(),
                "So many async, are you sure you're not writing JavaScript?".to_string(),
                "Async abuse! Suggest learning the beauty of synchronous programming".to_string(),
                "Too many async functions, careful not to confuse yourself".to_string(),
                "Using await more frequently than I wait for food delivery".to_string(),
                "So many awaits, what is your program waiting for? The apocalypse?".to_string(),
            ],
            ("en-US", "dyn-trait-abuse") => vec![
                "Using dyn traits more frequently than I change jobs".to_string(),
                "So much dynamic dispatch, where did the performance go?".to_string(),
                "Dyn trait abuse, are you sure you're not writing Python?".to_string(),
                "Too many dynamic traits, even compiler optimizations are crying".to_string(),
                "So many dyns, your program is more changeable than a chameleon".to_string(),
            ],
            ("en-US", "unsafe-abuse") => vec![
                "Unsafe code! Are you playing with fire or challenging Rust's bottom line?".to_string(),
                "Another unsafe! What is safety? Can you eat it?".to_string(),
                "Unsafe user, congratulations on earning the 'Memory Safety Destroyer' title".to_string(),
                "This unsafe reminds me of the horrifying memories of C language".to_string(),
                "Unsafe code: the existence that keeps Rust programmers awake at night".to_string(),
            ],
            ("en-US", "ffi-abuse") => vec![
                "FFI abuse! How many languages are you dating?".to_string(),
                "External interfaces are more complex than my social relationships!".to_string(),
                "So much FFI, Rust's safety is crying".to_string(),
                "Too many C interfaces, are you sure this is still a Rust project?".to_string(),
                "FFI code reminds me of the horror of pointer hell".to_string(),
            ],
            ("en-US", "macro-abuse") => vec![
                "More macro definitions than my excuses".to_string(),
                "So many macros, are you sure you're not writing C?".to_string(),
                "Macro abuse! You've made compile time longer".to_string(),
                "Too many macros, prepare to cry when debugging".to_string(),
                "So many macros, even the IDE wants to quit".to_string(),
            ],
            ("en-US", "module-complexity") => vec![
                "Module nesting deeper than Russian dolls".to_string(),
                "This module structure is more complex than my family relationships".to_string(),
                "Module nesting too deep, suggest reorganizing code structure".to_string(),
                "Such deep modules, finding a function is harder than finding treasure".to_string(),
            ],
            ("en-US", "pattern-matching-abuse") => vec![
                "Pattern matching more complex than my love life".to_string(),
                "So many patterns, are you writing a puzzle game?".to_string(),
                "Too many patterns, suggest simplifying logic".to_string(),
                "Complex patterns make code readability plummet".to_string(),
                "More match branches than my life choices".to_string(),
                "So many match branches, are you sure you're not writing a state machine?".to_string(),
            ],
            ("en-US", "reference-abuse") => vec![
                "References more complex than my social relationships".to_string(),
                "So many references, are you sure you're not writing a pointer maze?".to_string(),
                "Too many references, careful the borrow checker might strike".to_string(),
                "Reference count exceeded, suggest redesigning data structures".to_string(),
            ],
            ("en-US", "box-abuse") => vec![
                "Using Box more frequently than courier services".to_string(),
                "So many Boxes, are you opening a warehouse?".to_string(),
                "Too many Boxes, heap memory is about to explode".to_string(),
                "Box abuse, suggest considering stack allocation".to_string(),
                "So many Boxes, even the memory allocator is tired".to_string(),
            ],
            ("en-US", "slice-abuse") => vec![
                "Slicing more frequently than I chop vegetables".to_string(),
                "So many slices, are you opening a fruit shop?".to_string(),
                "Too many slices, you've chopped the arrays to pieces".to_string(),
                "Slice abuse, suggest using Vec instead".to_string(),
            ],
            ("en-US", "code-duplication") => vec![
                "Copy-paste ninja detected! 🥷 Your code has more duplicates than a hall of mirrors".to_string(),
                "DRY principle is crying in the corner while your code is drowning in repetition".to_string(),
                "This much duplication suggests you should rename your file to 'ctrl-c-ctrl-v.rs'".to_string(),
                "Duplicate code alert! Even my copy machine is jealous of your efficiency".to_string(),
                "Your code has more clones than a sci-fi movie, time for some refactoring!".to_string(),
            ],
            _ => vec!["Unknown issue detected".to_string()],
        }
    }
}
