// Internationalization (i18n) for Remote Messaging Gateways (Telegram, Discord)
// Standardized across 5 supported languages: pt-BR, en, es, zh-CN, ru

pub fn get_fresh_conversation_msg(lang: &str) -> &'static str {
    if lang.starts_with("pt") {
        "✨ **Nova conversa iniciada.** O contexto de memória foi redefinido para esta sessão."
    } else if lang.starts_with("es") {
        "✨ **Nueva conversación iniciada.** El contexto de memoria se ha restablecido para esta sesión."
    } else if lang.starts_with("zh") {
        "✨ **已开启全新对话。** 本次会话的记忆上下文已重置。"
    } else if lang.starts_with("ru") {
        "✨ **Начат новый диалог.** Контекст памяти был сброшен для этой сессии."
    } else {
        "✨ **Fresh conversation started.** Memory context has been reset for this session."
    }
}

pub fn get_help_msg(lang: &str) -> &'static str {
    if lang.starts_with("pt") {
        "👋 **Olá! Sou a Atena Studio.**\nConverse comigo diretamente por aqui!\n\n**Comandos disponíveis:**\n• `/start` — Iniciar conversa com a Atena\n• `/help` — Exibir lista de comandos disponíveis\n• `/new` — Iniciar nova conversa e redefinir contexto\n• `/model` — Ver o modelo de linguagem ativo\n• `/memory <termo>` — Pesquisar na memória cognitiva permanente\n• `/id` — Exibir seu ID numérico de usuário\n\nEnvie qualquer mensagem para conversar!"
    } else if lang.starts_with("es") {
        "👋 **¡Hola! Soy Atena Studio.**\n¡Chatea conmigo directamente desde aquí!\n\n**Comandos disponibles:**\n• `/start` — Iniciar conversación con Atena\n• `/help` — Mostrar comandos disponibles\n• `/new` — Iniciar nueva conversación y limpiar contexto\n• `/model` — Ver el modelo de lenguaje activo\n• `/memory <término>` — Buscar en la memoria cognitiva permanente\n• `/id` — Mostrar tu ID de usuario\n\n¡Envía cualquier mensaje para chatear!"
    } else if lang.starts_with("zh") {
        "👋 **你好！我是 Atena Studio。**\n直接在此与我对话！\n\n**可用命令：**\n• `/start` — 开始与 Atena 对话\n• `/help` — 查看可用命令列表\n• `/new` — 开启全新对话并重置上下文\n• `/model` — 查看当前激活的语言模型\n• `/memory <关键词>` — 检索持久认知记忆图谱\n• `/id` — 查看您的用户 ID\n\n直接发送任意消息即可开始交流！"
    } else if lang.starts_with("ru") {
        "👋 **Здравствуйте! Я Atena Studio.**\nОбщайтесь со мной прямо здесь!\n\n**Доступные команды:**\n• `/start` — Начать диалог с Atena\n• `/help` — Показать список команд\n• `/new` — Начать новый диалог и сбросить контекст\n• `/model` — Показать активную языковую модель\n• `/memory <запрос>` — Поиск по постоянной когнитивной памяти\n• `/id` — Показать ваш ID пользователя\n\nОтправьте любое сообщение, чтобы начать диалог!"
    } else {
        "👋 **Hello! I am Atena Studio.**\nChat with me directly from here!\n\n**Available commands:**\n• `/start` — Start chatting with Atena\n• `/help` — List available commands\n• `/new` — Start a fresh conversation and reset memory\n• `/model` — Show active language model\n• `/memory <query>` — Search long-term cognitive memory\n• `/id` — Show your numeric User ID\n\nSimply send any message to begin chatting!"
    }
}

pub fn get_access_restricted_msg(lang: &str, user_id: &str) -> String {
    if lang.starts_with("pt") {
        format!(
            "⚠️ **Acesso Restrito**\n\nSeu ID de usuário é `{}`.\nPara autorizar esta conta, adicione este ID em **Atena Studio → Configurações → Canais & Mensagens**.",
            user_id
        )
    } else if lang.starts_with("es") {
        format!(
            "⚠️ **Acceso Restringido**\n\nTu ID de usuario es `{}`.\nPara autorizar esta cuenta, añade este ID en **Atena Studio → Ajustes → Canales y Mensajería**.",
            user_id
        )
    } else if lang.starts_with("zh") {
        format!(
            "⚠️ **访问受限**\n\n您的用户 ID 为 `{}`。\n如需授权此账号，请在 **Atena Studio → 设置 → 远程网关** 中添加该 ID。",
            user_id
        )
    } else if lang.starts_with("ru") {
        format!(
            "⚠️ **Доступ ограничен**\n\nВаш ID пользователя: `{}`.\nЧтобы авторизовать этот аккаунт, добавьте этот ID в **Atena Studio → Настройки → Удаленные шлюзы**.",
            user_id
        )
    } else {
        format!(
            "⚠️ **Access Restricted**\n\nYour User ID is `{}`.\nTo authorize this account, add this ID in **Atena Studio → Settings → Remote Gateways**.",
            user_id
        )
    }
}

pub fn get_user_id_msg(lang: &str, user_id: &str) -> String {
    if lang.starts_with("pt") {
        format!("🆔 Seu ID de usuário é: **{}**", user_id)
    } else if lang.starts_with("es") {
        format!("🆔 Tu ID de usuario es: **{}**", user_id)
    } else if lang.starts_with("zh") {
        format!("🆔 您的用户 ID 为: **{}**", user_id)
    } else if lang.starts_with("ru") {
        format!("🆔 Ваш ID пользователя: **{}**", user_id)
    } else {
        format!("🆔 Your User ID is: **{}**", user_id)
    }
}

pub fn get_model_msg(lang: &str, model_name: &str) -> String {
    if lang.starts_with("pt") {
        format!("🤖 **Modelo Ativo:**\n`{}`", model_name)
    } else if lang.starts_with("es") {
        format!("🤖 **Modelo Activo:**\n`{}`", model_name)
    } else if lang.starts_with("zh") {
        format!("🤖 **当前激活模型:**\n`{}`", model_name)
    } else if lang.starts_with("ru") {
        format!("🤖 **Активная модель:**\n`{}`", model_name)
    } else {
        format!("🤖 **Active Model:**\n`{}`", model_name)
    }
}

pub fn get_no_model_msg(lang: &str) -> &'static str {
    if lang.starts_with("pt") {
        "🤖 Nenhum modelo está carregado no momento. Abra a Atena Studio para selecionar e carregar um modelo."
    } else if lang.starts_with("es") {
        "🤖 No hay ningún modelo cargado actualmente. Abra Atena Studio para seleccionar y cargar un modelo."
    } else if lang.starts_with("zh") {
        "🤖 当前未加载任何模型。请打开 Atena Studio 选择并加载模型。"
    } else if lang.starts_with("ru") {
        "🤖 В данный момент модель не загружена. Откройте Atena Studio для выбора и загрузки модели."
    } else {
        "🤖 No model currently loaded. Open Atena Studio to select and load a model."
    }
}

pub fn get_memory_usage_msg(lang: &str) -> &'static str {
    if lang.starts_with("pt") {
        "💡 Uso: `/memory <termos de busca>` (ex: `/memory café`, `/memory projeto`)"
    } else if lang.starts_with("es") {
        "💡 Uso: `/memory <términos de búsqueda>` (ej: `/memory café`, `/memory proyecto`)"
    } else if lang.starts_with("zh") {
        "💡 用法: `/memory <关键词>` (例如: `/memory 咖啡`, `/memory 项目`)"
    } else if lang.starts_with("ru") {
        "💡 Использование: `/memory <запрос>` (напр.: `/memory кофе`, `/memory проект`)"
    } else {
        "💡 Usage: `/memory <search terms>` (e.g. `/memory coffee`, `/memory project`)"
    }
}

pub fn get_memory_found_msg(lang: &str, query: &str, context: &str) -> String {
    if lang.starts_with("pt") {
        format!("🧠 **Memórias associativas para `{}`:**\n\n{}", query, context)
    } else if lang.starts_with("es") {
        format!("🧠 **Memorias asociativas para `{}`:**\n\n{}", query, context)
    } else if lang.starts_with("zh") {
        format!("🧠 **关于 `{}` 的联想记忆:**\n\n{}", query, context)
    } else if lang.starts_with("ru") {
        format!("🧠 **Ассоциативные воспоминания по `{}`:**\n\n{}", query, context)
    } else {
        format!("🧠 **Memories for `{}`:**\n\n{}", query, context)
    }
}

pub fn get_memory_not_found_msg(lang: &str, query: &str) -> String {
    if lang.starts_with("pt") {
        format!("🧠 Nenhuma memória associativa encontrada para: `{}`", query)
    } else if lang.starts_with("es") {
        format!("🧠 No se encontraron recuerdos asociativos para: `{}`", query)
    } else if lang.starts_with("zh") {
        format!("🧠 未找到关于 `{}` 的联想记忆", query)
    } else if lang.starts_with("ru") {
        format!("🧠 Ассоциативных воспоминаний не найдено для: `{}`", query)
    } else {
        format!("🧠 No associative memories found for: `{}`", query)
    }
}

pub fn get_default_system_prompt(lang: &str) -> &'static str {
    if lang.starts_with("pt") {
        "Você é a Atena Studio, uma assistente de IA inteligente, prestativa e amigável. Você se comunica sempre em português do Brasil com clareza, empatia e concisão. Quando o usuário solicitar tarefas, notícias, scripts ou comandos, acione as ferramentas disponíveis (<tool_call>) para obter os dados necessários e apresente o resultado final em linguagem natural."
    } else if lang.starts_with("es") {
        "Eres Atena Studio, una asistente de IA inteligente, servicial y amable. Te comunicas siempre en español con claridad, empatía y concisión. Cuando el usuario solicite tareas o scripts, invoca las herramientas disponibles (<tool_call>) y presenta el resultado en lenguaje natural."
    } else if lang.starts_with("zh") {
        "你是 Atena Studio，一个聪明、乐于助人且友善的 AI 助手。你始终以流畅清晰的中文进行交流。当用户请求任务或脚本时，调用可用工具 (<tool_call>) 并在最终回答中以自然语言呈现结果。"
    } else if lang.starts_with("ru") {
        "Вы — Atena Studio, умный, отзывчивый и дружелюбный ИИ-ассистент. Вы общаетесь на русском языке ясно и вежливо. При запросе задач или скриптов вызывайте доступные инструменты (<tool_call>) и представляйте результат пользователю."
    } else {
        "You are Atena Studio, an intelligent, helpful, and friendly AI assistant. You communicate clearly, warmly, and concisely in natural language. When the user requests tasks, scripts, or commands, invoke available tools (<tool_call>) and present the final results in natural language."
    }
}

pub fn get_tool_approval_prompt(lang: &str, tool_name: &str, args_summary: &str) -> String {
    if lang.starts_with("pt") {
        format!(
            "🛠️ **Permissão Solicitada para Ferramenta**\n\nA Atena precisa da sua autorização para executar:\n• **Ferramenta:** `{}`\n• **Parâmetros:**\n```json\n{}\n```\n\nDeseja autorizar a execução desta ferramenta?",
            tool_name, args_summary
        )
    } else if lang.starts_with("es") {
        format!(
            "🛠️ **Permiso Solicitado para Herramienta**\n\nAtena necesita tu autorización para ejecutar:\n• **Herramienta:** `{}`\n• **Parámetros:**\n```json\n{}\n```\n\n¿Deseas autorizar la ejecución de esta herramienta?",
            tool_name, args_summary
        )
    } else if lang.starts_with("zh") {
        format!(
            "🛠️ **工具调用权限请求**\n\nAtena 需要您的批准以执行以下工具：\n• **工具：** `{}`\n• **参数：**\n```json\n{}\n```\n\n是否批准执行此工具？",
            tool_name, args_summary
        )
    } else if lang.starts_with("ru") {
        format!(
            "🛠️ **Запрос разрешения на запуск инструмента**\n\nAtena запрашивает разрешение на выполнение:\n• **Инструмент:** `{}`\n• **Параметры:**\n```json\n{}\n```\n\nРазрешить выполнение этого инструмента?",
            tool_name, args_summary
        )
    } else {
        format!(
            "🛠️ **Tool Permission Request**\n\nAtena needs your authorization to execute:\n• **Tool:** `{}`\n• **Parameters:**\n```json\n{}\n```\n\nDo you want to authorize this tool execution?",
            tool_name, args_summary
        )
    }
}

pub fn get_tool_approval_btn_approve(lang: &str) -> &'static str {
    if lang.starts_with("pt") {
        "✅ Autorizar"
    } else if lang.starts_with("es") {
        "✅ Autorizar"
    } else if lang.starts_with("zh") {
        "✅ 批准执行"
    } else if lang.starts_with("ru") {
        "✅ Разрешить"
    } else {
        "✅ Authorize"
    }
}

pub fn get_tool_approval_btn_reject(lang: &str) -> &'static str {
    if lang.starts_with("pt") {
        "❌ Rejeitar"
    } else if lang.starts_with("es") {
        "❌ Rechazar"
    } else if lang.starts_with("zh") {
        "❌ 拒绝"
    } else if lang.starts_with("ru") {
        "❌ Отклонить"
    } else {
        "❌ Reject"
    }
}

pub fn get_tool_approved_msg(lang: &str, tool_name: &str) -> String {
    if lang.starts_with("pt") {
        format!("✅ **Ferramenta autorizada:** `{}`\n\n_Executando..._", tool_name)
    } else if lang.starts_with("es") {
        format!("✅ **Herramienta autorizada:** `{}`\n\n_Ejecutando..._", tool_name)
    } else if lang.starts_with("zh") {
        format!("✅ **工具已批准：** `{}`\n\n_正在执行..._", tool_name)
    } else if lang.starts_with("ru") {
        format!("✅ **Инструмент одобрен:** `{}`\n\n_Выполняется..._", tool_name)
    } else {
        format!("✅ **Tool authorized:** `{}`\n\n_Executing..._", tool_name)
    }
}

pub fn get_tool_rejected_msg(lang: &str, tool_name: &str) -> String {
    if lang.starts_with("pt") {
        format!("❌ **Ferramenta rejeitada pelo usuário:** `{}`", tool_name)
    } else if lang.starts_with("es") {
        format!("❌ **Herramienta rechazada por el usuario:** `{}`", tool_name)
    } else if lang.starts_with("zh") {
        format!("❌ **用户已拒绝工具执行：** `{}`", tool_name)
    } else if lang.starts_with("ru") {
        format!("❌ **Инструмент отклонен пользователем:** `{}`", tool_name)
    } else {
        format!("❌ **Tool rejected by user:** `{}`", tool_name)
    }
}

pub fn get_tool_executing_toast(lang: &str) -> &'static str {
    if lang.starts_with("pt") {
        "Executando ferramenta..."
    } else if lang.starts_with("es") {
        "Ejecutando herramienta..."
    } else if lang.starts_with("zh") {
        "正在执行工具..."
    } else if lang.starts_with("ru") {
        "Выполняется инструмент..."
    } else {
        "Executing tool..."
    }
}

pub fn get_tool_rejected_toast(lang: &str) -> &'static str {
    if lang.starts_with("pt") {
        "Execução cancelada."
    } else if lang.starts_with("es") {
        "Ejecución cancelada."
    } else if lang.starts_with("zh") {
        "已取消执行。"
    } else if lang.starts_with("ru") {
        "Выполнение отменено."
    } else {
        "Execution cancelled."
    }
}

pub fn get_tool_user_rejected_prompt(lang: &str, tool_name: &str) -> String {
    if lang.starts_with("pt") {
        format!(
            "O usuário recusou a execução da ferramenta '{}'. Avise educadamente que a ação não foi realizada e pergunte como prefere prosseguir.",
            tool_name
        )
    } else if lang.starts_with("es") {
        format!(
            "El usuario rechazó la ejecución de la herramienta '{}'. Notifica amablemente que no se realizó la acción y consulta cómo prefiere continuar.",
            tool_name
        )
    } else if lang.starts_with("zh") {
        format!(
            "用户拒绝了工具 '{}' 的执行。请礼貌告知操作未执行，并询问用户希望如何继续。",
            tool_name
        )
    } else if lang.starts_with("ru") {
        format!(
            "Пользователь отклонил запуск инструмента '{}'. Вежливо сообщите, что действие не выполнено, и уточните, как продолжить.",
            tool_name
        )
    } else {
        format!(
            "The user rejected the execution of tool '{}'. Politely inform them that the action was not taken and ask how they would like to proceed.",
            tool_name
        )
    }
}

pub fn get_tool_results_prompt(lang: &str, tool_results_block: &str) -> String {
    if lang.starts_with("pt") {
        format!(
            "[Retorno das Ferramentas Executadas]:\n{}\n\nPor favor, apresente o resultado final diretamente ao usuário de forma clara e objetiva.",
            tool_results_block
        )
    } else if lang.starts_with("es") {
        format!(
            "[Resultado de las Herramientas Ejecutadas]:\n{}\n\nPor favor, presenta el resultado final directamente al usuario de forma clara y objetiva.",
            tool_results_block
        )
    } else if lang.starts_with("zh") {
        format!(
            "[已执行工具返回结果]：\n{}\n\n请用清晰明了的语言直接向用户呈现最终结果。",
            tool_results_block
        )
    } else if lang.starts_with("ru") {
        format!(
            "[Результаты выполнения инструментов]:\n{}\n\nПожалуйста, четко и ясно представьте окончательный ответ пользователю.",
            tool_results_block
        )
    } else {
        format!(
            "[Tool Execution Results]:\n{}\n\nPlease present the final result directly to the user clearly and objectively.",
            tool_results_block
        )
    }
}

