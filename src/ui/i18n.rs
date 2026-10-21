//! Lightweight embedded internationalization system.
//! Supports English (en), Français (fr), Español (es), and 简体中文 (zh-CN).
//! Zero external runtime dependencies.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Locale {
    En,
    Fr,
    Es,
    ZhCn,
}

pub const ALL_LOCALES: &[Locale] = &[Locale::En, Locale::Fr, Locale::Es, Locale::ZhCn];

impl Locale {
    pub fn as_code(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Fr => "fr",
            Locale::Es => "es",
            Locale::ZhCn => "zh-CN",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::Fr => "Français",
            Locale::Es => "Español",
            Locale::ZhCn => "简体中文",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        let lower = code.trim().to_lowercase();
        if lower == "en" || lower.starts_with("en_") || lower.starts_with("en-") {
            Some(Locale::En)
        } else if lower == "fr" || lower.starts_with("fr_") || lower.starts_with("fr-") {
            Some(Locale::Fr)
        } else if lower == "es" || lower.starts_with("es_") || lower.starts_with("es-") {
            Some(Locale::Es)
        } else if lower == "zh"
            || lower == "zh-cn"
            || lower == "zh_cn"
            || lower.starts_with("zh-")
            || lower.starts_with("zh_")
        {
            Some(Locale::ZhCn)
        } else {
            None
        }
    }

    /// Automatically detects user's preferred locale from environment or OS settings.
    pub fn detect() -> Self {
        if let Ok(val) = std::env::var("GALERNE_LANG") {
            if let Some(loc) = Self::from_code(&val) {
                return loc;
            }
        }

        for var in &["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(val) = std::env::var(var) {
                if let Some(loc) = Self::from_code(&val) {
                    return loc;
                }
            }
        }

        #[cfg(windows)]
        {
            // Simple registry/env check on Windows
            if let Ok(val) = std::env::var("LANG") {
                if let Some(loc) = Self::from_code(&val) {
                    return loc;
                }
            }
        }

        Locale::En
    }
}

pub const ALL_KEYS: &[&str] = &[
    "menu_play",
    "menu_workshop",
    "menu_stats",
    "menu_saves",
    "menu_doctor",
    "menu_settings",
    "menu_help",
    "menu_exit",
    "deploy_operation",
    "resume_operation",
    "active_run_desc",
    "press_enter_resume",
    "controls_hint",
    "workshop_title",
    "workshop_subtitle",
    "base_health",
    "base_health_desc",
    "move_speed",
    "move_speed_desc",
    "magnet_radius",
    "magnet_radius_desc",
    "shard_mult",
    "shard_mult_desc",
    "dash_cooldown",
    "dash_cooldown_desc",
    "cost",
    "level",
    "max_level",
    "insufficient_shards",
    "stats_title",
    "play_time",
    "wait_saved",
    "runs_started",
    "distance_traveled",
    "shards_collected",
    "settings_title",
    "color_effects",
    "animations",
    "auto_activate",
    "language",
    "anti_flicker_delay",
    "doctor_title",
    "system_healthy",
    "executable",
    "target_platform",
    "standard_output",
    "terminal_size",
    "color_depth",
    "data_storage",
    "save_integrity",
    "shell_hook",
    "resident_daemons",
    "network_policy",
    "jump_prompt",
    "slide_prompt",
    "dash_prompt",
    "hud_distance",
    "hud_speed",
    "hud_multiplier",
    "hud_shards",
];

pub fn t(key: &'static str, locale: Locale) -> &'static str {
    match locale {
        Locale::En => translate_en(key),
        Locale::Fr => translate_fr(key).unwrap_or_else(|| translate_en(key)),
        Locale::Es => translate_es(key).unwrap_or_else(|| translate_en(key)),
        Locale::ZhCn => translate_zh(key).unwrap_or_else(|| translate_en(key)),
    }
}

fn translate_en(key: &'static str) -> &'static str {
    match key {
        "menu_play" => "PLAY",
        "menu_workshop" => "WORKSHOP",
        "menu_stats" => "STATS",
        "menu_saves" => "SAVES",
        "menu_doctor" => "DOCTOR",
        "menu_settings" => "SETTINGS",
        "menu_help" => "HELP",
        "menu_exit" => "EXIT",
        "deploy_operation" => "DEPLOYMENT CONTROLS",
        "resume_operation" => "Press Enter to Resume Operation",
        "active_run_desc" => "Active Operation in Progress",
        "press_enter_resume" => "Press Enter to Deploy into Wind Sector",
        "controls_hint" => "↑/↓: navigate menu   ←/→: workshop items   Enter: select   Esc/q: exit",
        "workshop_title" => "ENGINEERING WORKSHOP",
        "workshop_subtitle" => "Upgrade persistent windrunner modules with Gale Shards",
        "base_health" => "Kinetic Shield",
        "base_health_desc" => "Absorbs obstacle impacts before failing",
        "move_speed" => "Forward Momentum",
        "move_speed_desc" => "Increases baseline cruising speed",
        "magnet_radius" => "Shard Collector",
        "magnet_radius_desc" => "Magnetic pull radius for nearby shards",
        "shard_mult" => "Shard Synthesizer",
        "shard_mult_desc" => "Increases shards yielded per pickup",
        "dash_cooldown" => "Thruster Cooldown",
        "dash_cooldown_desc" => "Reduces recovery time between dashes",
        "cost" => "Cost",
        "level" => "Level",
        "max_level" => "MAX",
        "insufficient_shards" => "Insufficient Shards",
        "stats_title" => "CAREER TELEMETRY",
        "play_time" => "Total Active Play Time",
        "wait_saved" => "Developer Wait Time Saved",
        "runs_started" => "Operations Initiated",
        "distance_traveled" => "Distance Traveled",
        "shards_collected" => "Lifetime Shards Harvested",
        "settings_title" => "OPERATIONAL SETTINGS",
        "color_effects" => "ANSI Color Palette",
        "animations" => "Ambient Visual Effects",
        "auto_activate" => "Auto-Activate on Builds",
        "language" => "Interface Language",
        "anti_flicker_delay" => "Anti-Flicker Delay",
        "doctor_title" => "SYSTEM DIAGNOSTICS",
        "system_healthy" => "All operational invariants verified healthy.",
        "executable" => "Executable",
        "target_platform" => "Target Platform",
        "standard_output" => "Standard Output",
        "terminal_size" => "Terminal Size",
        "color_depth" => "Color Depth",
        "data_storage" => "Data Storage",
        "save_integrity" => "Save Integrity",
        "shell_hook" => "Shell Hook",
        "resident_daemons" => "Resident Daemons",
        "network_policy" => "Network Policy",
        "jump_prompt" => "SPACE / UP : JUMP",
        "slide_prompt" => "DOWN / S : SLIDE",
        "dash_prompt" => "X / RIGHT : DASH",
        "hud_distance" => "DIST",
        "hud_speed" => "SPD",
        "hud_multiplier" => "FLOW",
        "hud_shards" => "SHARDS",
        _ => key,
    }
}

fn translate_fr(key: &str) -> Option<&'static str> {
    Some(match key {
        "menu_play" => "JOUER",
        "menu_workshop" => "ATELIER",
        "menu_stats" => "STATS",
        "menu_saves" => "SAUVEGARDES",
        "menu_doctor" => "DIAGNOSTIC",
        "menu_settings" => "RÉGLAGES",
        "menu_help" => "AIDE",
        "menu_exit" => "QUITTER",
        "deploy_operation" => "CONTRÔLES DE DÉPLOIEMENT",
        "resume_operation" => "Appuyez sur Entrée pour reprendre",
        "active_run_desc" => "Opération en cours d'exécution",
        "press_enter_resume" => "Appuyez sur Entrée pour vous déployer",
        "controls_hint" => "↑/↓: naviguer   ←/→: modules   Entrée: valider   Échap/q: quitter",
        "workshop_title" => "ATELIER D'INGÉNIERIE",
        "workshop_subtitle" => "Améliorez vos modules avec des Éclats de Galerne",
        "base_health" => "Bouclier Cinétique",
        "base_health_desc" => "Absorbe les chocs d'obstacles",
        "move_speed" => "Élan Vers l'Avant",
        "move_speed_desc" => "Augmente la vitesse de croisière",
        "magnet_radius" => "Collecteur d'Éclats",
        "magnet_radius_desc" => "Rayon d'attraction des éclats proches",
        "shard_mult" => "Synthétiseur d'Éclats",
        "shard_mult_desc" => "Multiplie le rendement des éclats",
        "dash_cooldown" => "Recharge Propulseur",
        "dash_cooldown_desc" => "Diminue le temps de recharge du sprint",
        "cost" => "Coût",
        "level" => "Niveau",
        "max_level" => "MAX",
        "insufficient_shards" => "Éclats insuffisants",
        "stats_title" => "TÉLÉMÉTRIE DE CARRIÈRE",
        "play_time" => "Temps de jeu total",
        "wait_saved" => "Temps d'attente absorbé",
        "runs_started" => "Opérations lancées",
        "distance_traveled" => "Distance parcourue",
        "shards_collected" => "Éclats totaux récoltés",
        "settings_title" => "PARAMÈTRES SYSTÈME",
        "color_effects" => "Palette de couleurs ANSI",
        "animations" => "Effets visuels ambiants",
        "auto_activate" => "Activation auto sur build",
        "language" => "Langue de l'interface",
        "anti_flicker_delay" => "Délai anti-scintillement",
        "doctor_title" => "DIAGNOSTIC DU SYSTÈME",
        "system_healthy" => "Tous les invariants sont opérationnels.",
        "executable" => "Exécutable",
        "target_platform" => "Plateforme cible",
        "standard_output" => "Sortie standard",
        "terminal_size" => "Taille terminal",
        "color_depth" => "Profondeur couleur",
        "data_storage" => "Stockage données",
        "save_integrity" => "Intégrité sauvegarde",
        "shell_hook" => "Hook shell",
        "resident_daemons" => "Démons résidents",
        "network_policy" => "Politique réseau",
        "jump_prompt" => "ESPACE / HAUT : SAUTER",
        "slide_prompt" => "BAS / S : GLISSER",
        "dash_prompt" => "X / DROITE : SPRINT",
        "hud_distance" => "DIST",
        "hud_speed" => "VIT",
        "hud_multiplier" => "ÉLAN",
        "hud_shards" => "ÉCLATS",
        _ => return None,
    })
}

fn translate_es(key: &str) -> Option<&'static str> {
    Some(match key {
        "menu_play" => "JUGAR",
        "menu_workshop" => "TALLER",
        "menu_stats" => "ESTADÍSTICAS",
        "menu_saves" => "PARTIDAS",
        "menu_doctor" => "DIAGNÓSTICO",
        "menu_settings" => "AJUSTES",
        "menu_help" => "AYUDA",
        "menu_exit" => "SALIR",
        "deploy_operation" => "CONTROLES DE DESPLIEGUE",
        "resume_operation" => "Pulsa Enter para reanudar",
        "active_run_desc" => "Operación activa en curso",
        "press_enter_resume" => "Pulsa Enter para desplegarte",
        "controls_hint" => "↑/↓: navegar   ←/→: módulos   Enter: seleccionar   Esc/q: salir",
        "workshop_title" => "TALLER DE INGENIERÍA",
        "workshop_subtitle" => "Mejora tus módulos con Fragmentos de Galerna",
        "base_health" => "Escudo Cinético",
        "base_health_desc" => "Absorbe impactos de obstáculos",
        "move_speed" => "Impulso Frontal",
        "move_speed_desc" => "Aumenta la velocidad de avance",
        "magnet_radius" => "Colector de Fragmentos",
        "magnet_radius_desc" => "Radio de atracción de fragmentos",
        "shard_mult" => "Sintetizador",
        "shard_mult_desc" => "Aumenta los fragmentos obtenidos",
        "dash_cooldown" => "Recarga de Propulsor",
        "dash_cooldown_desc" => "Reduce el tiempo de recarga del sprint",
        "cost" => "Coste",
        "level" => "Nivel",
        "max_level" => "MÁX",
        "insufficient_shards" => "Fragmentos insuficientes",
        "stats_title" => "TELEMETRÍA DE CARRERA",
        "play_time" => "Tiempo total de juego",
        "wait_saved" => "Tiempo de espera absorbido",
        "runs_started" => "Operaciones iniciadas",
        "distance_traveled" => "Distancia recorrida",
        "shards_collected" => "Total de fragmentos",
        "settings_title" => "AJUSTES DE SISTEMA",
        "color_effects" => "Paleta de colores ANSI",
        "animations" => "Efectos visuales",
        "auto_activate" => "Activar con compilaciones",
        "language" => "Idioma de interfaz",
        "anti_flicker_delay" => "Retardo anti-parpadeo",
        "doctor_title" => "DIAGNÓSTICO DEL SISTEMA",
        "system_healthy" => "Todos los invariantes verificados.",
        "executable" => "Ejecutable",
        "target_platform" => "Plataforma objetivo",
        "standard_output" => "Salida estándar",
        "terminal_size" => "Tamaño terminal",
        "color_depth" => "Profundidad color",
        "data_storage" => "Almacenamiento",
        "save_integrity" => "Integridad partida",
        "shell_hook" => "Hook de shell",
        "resident_daemons" => "Servicios en segundo plano",
        "network_policy" => "Política de red",
        "jump_prompt" => "ESPACIO / ARRIBA : SALTAR",
        "slide_prompt" => "ABAJO / S : DESLIZAR",
        "dash_prompt" => "X / DERECHA : SPRINT",
        "hud_distance" => "DIST",
        "hud_speed" => "VEL",
        "hud_multiplier" => "FLUJO",
        "hud_shards" => "FRAGMENTOS",
        _ => return None,
    })
}

fn translate_zh(key: &str) -> Option<&'static str> {
    Some(match key {
        "menu_play" => "开始行动",
        "menu_workshop" => "工坊",
        "menu_stats" => "统计",
        "menu_saves" => "存档",
        "menu_doctor" => "诊断",
        "menu_settings" => "设置",
        "menu_help" => "帮助",
        "menu_exit" => "退出",
        "deploy_operation" => "行动控制中心",
        "resume_operation" => "按回车继续行动",
        "active_run_desc" => "进行中的行动",
        "press_enter_resume" => "按回车键部署至风场区域",
        "controls_hint" => "↑/↓: 导航   ←/→: 工坊模块   回车: 选择   Esc/q: 退出",
        "workshop_title" => "工程研制工坊",
        "workshop_subtitle" => "使用加莱尔风能碎片强化核心奔跑模块",
        "base_health" => "动能护盾",
        "base_health_desc" => "吸收撞击伤害",
        "move_speed" => "基准航速",
        "move_speed_desc" => "提升常规向前奔跑速度",
        "magnet_radius" => "引力收集器",
        "magnet_radius_desc" => "扩大吸附风能碎片的半径",
        "shard_mult" => "碎片合成器",
        "shard_mult_desc" => "提升碎片拾取收益",
        "dash_cooldown" => "冲刺推进冷却",
        "dash_cooldown_desc" => "缩短突进冲刺的冷却时间",
        "cost" => "消耗",
        "level" => "等级",
        "max_level" => "满级",
        "insufficient_shards" => "碎片不足",
        "stats_title" => "生涯遥测数据",
        "play_time" => "总游玩时间",
        "wait_saved" => "转化的编译等待时间",
        "runs_started" => "部署行动总次数",
        "distance_traveled" => "累计奔跑距离",
        "shards_collected" => "生涯获取碎片总量",
        "settings_title" => "系统运行设置",
        "color_effects" => "ANSI色彩渲染",
        "animations" => "环境视觉特效",
        "auto_activate" => "构建任务自动唤醒",
        "language" => "界面语言",
        "anti_flicker_delay" => "防闪烁延迟",
        "doctor_title" => "系统运行诊断",
        "system_healthy" => "所有系统运行不变量校验正常。",
        "executable" => "执行程序",
        "target_platform" => "目标平台",
        "standard_output" => "终端输出",
        "terminal_size" => "终端尺寸",
        "color_depth" => "色彩深度",
        "data_storage" => "本地存储",
        "save_integrity" => "存档完整性",
        "shell_hook" => "Shell集成",
        "resident_daemons" => "后台守护进程",
        "network_policy" => "网络策略",
        "jump_prompt" => "空格 / 上箭头 : 跳跃",
        "slide_prompt" => "下箭头 / S : 滑铲",
        "dash_prompt" => "X / 右箭头 : 冲刺",
        "hud_distance" => "距离",
        "hud_speed" => "速度",
        "hud_multiplier" => "心流",
        "hud_shards" => "碎片",
        _ => return None,
    })
}
