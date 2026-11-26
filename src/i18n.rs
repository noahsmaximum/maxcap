//! English / French interface text.
//!
//! The English text in the code is the key. `tr` looks it up (ignoring punctuation,
//! spacing and case, so a dash or an ellipsis never breaks a match) and falls back to
//! the English text when there is no French entry.

use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

static FRENCH: AtomicBool = AtomicBool::new(true);

pub fn set(french: bool) {
    FRENCH.store(french, Ordering::Relaxed);
}

pub fn is_fr() -> bool {
    FRENCH.load(Ordering::Relaxed)
}

fn norm(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_alphanumeric()).map(|c| c.to_ascii_lowercase()).collect()
}

fn map() -> &'static HashMap<String, &'static str> {
    static MAP: OnceLock<HashMap<String, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| TABLE.iter().map(|(en, fr)| (norm(en), *fr)).collect())
}

/// Translate a fixed UI string.
pub fn tr(s: &str) -> &str {
    if !is_fr() {
        return s;
    }
    let key = norm(s);
    if key.is_empty() {
        return s;
    }
    map().get(&key).copied().unwrap_or(s)
}

/// Translate a message that may carry a variable tail ("Saved preset X").
pub fn tr_dyn(s: &str) -> String {
    if !is_fr() {
        return s.to_owned();
    }
    let t = tr(s);
    if t != s {
        return t.to_owned();
    }
    for (pre, fr) in PREFIXES {
        if let Some(rest) = s.strip_prefix(pre) {
            return format!("{fr}{rest}");
        }
    }
    for (verb, fr) in [("Exported ", "Exporté : "), ("Imported ", "Importé : ")] {
        if let Some(n) = s.strip_prefix(verb).and_then(|r| r.strip_suffix(" profiles")) {
            return format!("{fr}{n} profils");
        }
    }
    s.to_owned()
}

const PREFIXES: &[(&str, &str)] = &[
    ("Saved preset ", "Preset enregistré : "),
    ("Updated preset ", "Preset mis à jour : "),
    ("Couldn't save preset: ", "Échec de l'enregistrement : "),
    ("Couldn't update preset: ", "Échec de la mise à jour : "),
    ("Couldn't delete: ", "Échec de la suppression : "),
    ("Preset: ", "Preset : "),
    ("Saved ", "Enregistré : "),
    ("No preset in slot ", "Aucun preset à l'emplacement "),
    ("Export failed: ", "Échec de l'export : "),
    ("Import failed: ", "Échec de l'import : "),
    ("Linked preset ", "Preset lié introuvable : "),
    ("Audio: ", "Audio : "),
];

const TABLE: &[(&str, &str)] = &[
    // --- state tags, header, footer
    ("live", "en direct"),
    ("opening", "ouverture"),
    ("error", "erreur"),
    ("idle", "inactif"),
    ("esc", "échap"),
    ("Close (F1 / Tab / Esc)", "Fermer (F1 / Tab / Échap)"),
    ("windowed", "fenêtré"),
    ("Exit fullscreen (F11 / Esc)", "Quitter le plein écran (F11 / Échap)"),
    ("fullscreen", "plein écran"),
    ("Fullscreen (F11 / Alt+Enter)", "Plein écran (F11 / Alt+Entrée)"),
    ("low-latency capture viewer // v", "visionneuse de capture faible latence // v"),
    (
        "F1 menu · F2 stats · F11 full · ^S shot · M mute · P pin",
        "F1 menu · F2 stats · F11 plein écran · ^S capture · M muet · P épingler",
    ),
    // --- source
    ("Source", "Source"),
    ("Device", "Carte"),
    ("scan", "scanner"),
    ("Rescan devices", "Rechercher à nouveau les cartes"),
    ("Format", "Format"),
    ("Auto (best quality)", "Auto (meilleure qualité)"),
    ("uncompressed", "non compressé"),
    ("windows 4:2:0", "windows 4:2:0"),
    ("mjpeg 4:2:2", "mjpeg 4:2:2"),
    ("Uncompressed - best quality, lowest latency.", "Non compressé : meilleure qualité, latence minimale."),
    (
        "MJPEG decoded by Windows to 4:2:0 - pick the MJPEG mode for 4:2:2 colour.",
        "MJPEG décodé par Windows en 4:2:0 : choisis le mode MJPEG pour des couleurs 4:2:2.",
    ),
    (
        "MJPEG (lossy), decoded here to 4:2:2. Uncompressed modes are limited by USB bandwidth.",
        "MJPEG (avec pertes), décodé ici en 4:2:2. Les modes non compressés sont limités par la bande passante USB.",
    ),
    // --- picture
    ("Picture", "Image"),
    ("Auto", "Auto"),
    ("Matrix", "Matrice"),
    ("BT.601 (SD)", "BT.601 (SD)"),
    ("BT.709 (HD)", "BT.709 (HD)"),
    ("BT.2020 (UHD)", "BT.2020 (UHD)"),
    ("Range", "Plage"),
    ("Limited (16-235)", "Limitée (16-235)"),
    ("Full (0-255)", "Complète (0-255)"),
    ("Chroma filter", "Filtre chroma"),
    ("Bicubic (sharpest)", "Bicubique (le plus net)"),
    ("Bilinear", "Bilinéaire"),
    ("Nearest", "Plus proche voisin"),
    ("Chroma siting", "Position chroma"),
    ("Left / co-sited", "Gauche / co-localisée"),
    ("Center", "Centre"),
    ("full", "complète"),
    ("limited", "limitée"),
    ("left", "gauche"),
    ("Brightness", "Luminosité"),
    ("Contrast", "Contraste"),
    ("Saturation", "Saturation"),
    ("Hue", "Teinte"),
    ("Gamma", "Gamma"),
    ("Sharpen (CAS)", "Netteté (CAS)"),
    ("reset picture", "réinitialiser l'image"),
    // --- scaling
    ("Scaling", "Mise à l'échelle"),
    ("Filter", "Filtre"),
    ("Nearest (pixel exact)", "Plus proche voisin (pixel exact)"),
    ("Sharp bilinear (pixel art)", "Bilinéaire net (pixel art)"),
    ("Bicubic Catmull-Rom", "Bicubique Catmull-Rom"),
    ("Bicubic Mitchell", "Bicubique Mitchell"),
    ("AI upscale", "Upscale IA"),
    ("Off", "Désactivé"),
    ("NVIDIA RTX Video Super Resolution", "NVIDIA RTX Video Super Resolution"),
    ("Upscale to", "Upscale vers"),
    ("Match window", "Taille de la fenêtre"),
    ("2x source", "2x la source"),
    ("3x source", "3x la source"),
    ("4x source", "4x la source"),
    ("Quality", "Qualité"),
    ("Low", "Basse"),
    ("Medium", "Moyenne"),
    ("High", "Haute"),
    ("Ultra", "Ultra"),
    (
        "The filter below fits the upscaled picture to the window (1:1 in Match window).",
        "Le filtre ci-dessous ajuste l'image upscalée à la fenêtre (1:1 avec « Taille de la fenêtre »).",
    ),
    ("Frame gen", "Gén. d'images"),
    ("Motion smoothing (NVIDIA)", "Lissage du mouvement (NVIDIA)"),
    ("Multiplier", "Multiplicateur"),
    ("Fit display", "Adapter à l'écran"),
    ("Artifact guard", "Protection anti-artefacts"),
    (
        "Inserts 1-3 generated frames between each pair; the real frame shows up to a frame later (half at 2x, three quarters at 4x). For watching, not twitchy play. Fit display picks the most your refresh rate can show. With AI upscale, every frame is upscaled, so use a lower quality at 3-4x.",
        "Insère 1 à 3 images générées entre chaque paire ; l'image réelle s'affiche jusqu'à une image plus tard (la moitié à 2x, trois quarts à 4x). Pour regarder, pas pour jouer à des jeux très nerveux. « Adapter à l'écran » choisit le maximum que ta fréquence d'écran peut afficher. Avec l'upscale IA, chaque image est upscalée : choisis une qualité plus basse à 3-4x.",
    ),
    ("Anti-ringing", "Anti-ringing"),
    ("Scale in linear light", "Échelle en lumière linéaire"),
    // --- geometry
    ("Size", "Taille"),
    ("Fit (letterbox)", "Ajusté (bandes noires)"),
    ("Fill (crop)", "Rempli (recadré)"),
    ("Stretch", "Étiré"),
    ("Integer scale", "Échelle entière"),
    ("1:1 native", "1:1 natif"),
    ("Aspect ratio", "Format d'image"),
    ("Source (square pixels)", "Source (pixels carrés)"),
    ("Zoom", "Zoom"),
    ("Crop L / R", "Recadrage G / D"),
    ("Crop T / B", "Recadrage H / B"),
    ("Rotate", "Rotation"),
    ("Flip", "Miroir"),
    ("Horizontal", "Horizontal"),
    ("Vertical", "Vertical"),
    ("reset geometry", "réinitialiser la géométrie"),
    // --- presets
    ("Presets", "Presets"),
    ("Preset", "Preset"),
    ("No presets yet", "Aucun preset pour l'instant"),
    ("modified", "modifié"),
    ("saved", "enregistré"),
    ("Save as", "Enregistrer sous"),
    ("Preset name", "Nom du preset"),
    ("save", "enregistrer"),
    ("update", "mettre à jour"),
    ("sure?", "sûr ?"),
    ("delete", "supprimer"),
    ("folder", "dossier"),
    (
        "Open the presets folder. Renamed, copied or deleted files show up here automatically.",
        "Ouvre le dossier des presets. Les fichiers renommés, copiés ou supprimés apparaissent ici automatiquement.",
    ),
    ("Share", "Partage"),
    ("export all", "tout exporter"),
    ("Save every preset into one .json file", "Enregistre tous les presets dans un seul fichier .json"),
    ("import", "importer"),
    (
        "Add presets from a .json file (existing names are kept)",
        "Ajoute les presets d'un fichier .json (les noms existants sont conservés)",
    ),
    ("Auto for card", "Auto pour la carte"),
    ("linked", "lié"),
    ("unlink", "délier"),
    ("use for this card", "utiliser pour cette carte"),
    ("now:", "actuel :"),
    (
        "Presets keep picture, scaling, frame, display and audio processing - not which devices you use. Ctrl+1...9 loads them.",
        "Les presets gardent l'image, la mise à l'échelle, le cadre, l'affichage et le traitement audio, mais pas les appareils utilisés. Ctrl+1…9 les charge.",
    ),
    // built-in profiles
    ("Everything", "Tout"),
    ("FPS only", "FPS seul"),
    ("FPS + latency", "FPS + latence"),
    ("Gaming", "Jeu"),
    ("Diagnostics", "Diagnostic"),
    ("Custom", "Perso"),
    // --- background & frame
    ("Background & frame", "Fond et cadre"),
    ("Background", "Fond"),
    ("Solid colour", "Couleur unie"),
    ("Image", "Image"),
    ("Ambient (from the feed)", "Ambiance (depuis le flux)"),
    ("Style", "Style"),
    ("Glow", "Halo"),
    ("Streaks (waterfall)", "Traînées (cascade)"),
    ("Mirror", "Miroir"),
    ("Softness", "Douceur"),
    ("Reach", "Portée"),
    ("forever", "infinie"),
    ("Flow", "Flux"),
    (
        "Fills the bars around the picture from the picture itself. Shows when the window and video shapes differ, or with a margin.",
        "Remplit les bandes autour de l'image à partir de l'image elle-même. Visible quand la fenêtre et la vidéo n'ont pas la même forme, ou avec une marge.",
    ),
    ("browse", "parcourir"),
    ("Fit", "Ajustement"),
    ("Cover (fill, crop)", "Couvrir (remplir, recadrer)"),
    ("Contain", "Contenir"),
    ("Center (1:1)", "Centré (1:1)"),
    ("Tile", "Mosaïque"),
    ("Dim", "Assombrir"),
    ("Colour", "Couleur"),
    ("Frame", "Cadre"),
    ("None", "Aucun"),
    ("Outline", "Contour"),
    ("Pixel card", "Carte pixel"),
    ("Hard shadow", "Ombre dure"),
    ("Image (9-slice)", "Image (9 zones)"),
    ("Frame image", "Image du cadre"),
    ("Corner size", "Taille des coins"),
    (
        "Size of the frame image's corners. They stay sharp; the edges stretch.",
        "Taille des coins de l'image du cadre. Ils restent nets ; les bords s'étirent.",
    ),
    ("Thickness", "Épaisseur"),
    ("Overlap", "Chevauchement"),
    (
        "Pull the frame over the video edge (hides rounded inner corners).",
        "Fait déborder le cadre sur le bord de la vidéo (cache les coins intérieurs arrondis).",
    ),
    ("Frame colour", "Couleur du cadre"),
    ("Card colours", "Couleurs de la carte"),
    ("Band", "Bande"),
    ("Edge lines", "Traits de bord"),
    ("lines", "traits"),
    ("shadow", "ombre"),
    ("default colours", "couleurs par défaut"),
    ("Margin", "Marge"),
    (
        "Extra space around the video so more background shows.",
        "Espace supplémentaire autour de la vidéo pour laisser voir plus de fond.",
    ),
    ("Loading frame...", "Chargement du cadre…"),
    (
        "Frames wrap the video at every size and aspect mode; the video shrinks to make room.",
        "Les cadres entourent la vidéo à toutes les tailles et tous les formats ; la vidéo rétrécit pour leur faire de la place.",
    ),
    // --- display
    ("Display & latency", "Affichage et latence"),
    ("Present", "Présentation"),
    ("Immediate (lowest latency)", "Immédiat (latence minimale)"),
    ("Mailbox (no tearing)", "Mailbox (sans déchirure)"),
    ("V-Sync (smoothest)", "V-Sync (le plus fluide)"),
    ("Frame queue", "File d'images"),
    ("Output depth", "Profondeur de sortie"),
    ("Dither output", "Dithering de sortie"),
    ("Always on top", "Toujours au premier plan"),
    ("Ghost mode", "Mode fantôme"),
    ("exit ghost", "quitter fantôme"),
    ("enter ghost", "entrer en fantôme"),
    ("Ghost opacity", "Opacité fantôme"),
    (
        "Ghost: borderless, on top, see-through, clicks pass through. Leave with Ctrl+Alt+G (works from any app).",
        "Fantôme : sans bordure, au premier plan, translucide, les clics passent à travers. Quitte avec Ctrl+Alt+G (marche depuis n'importe quelle appli).",
    ),
    ("Start fullscreen", "Démarrer en plein écran"),
    ("Hide cursor", "Masquer le curseur"),
    (
        "Lowest latency: Immediate + 1 frame in flight + fullscreen (the frame skips the compositor).",
        "Latence minimale : Immédiat + 1 image en vol + plein écran (l'image évite le compositeur).",
    ),
    // --- audio
    ("Audio", "Audio"),
    ("Play capture audio", "Lire l'audio de la capture"),
    ("Input", "Entrée"),
    ("Auto (match video)", "Auto (selon la vidéo)"),
    ("Output", "Sortie"),
    ("System default", "Défaut du système"),
    ("Volume", "Volume"),
    ("Mute", "Muet"),
    ("Buffer / delay", "Tampon / délai"),
    ("MS2109 fix (96k mono -> 48k stereo)", "Correctif MS2109 (96k mono → 48k stéréo)"),
    (
        "Cheap MacroSilicon cards mislabel their audio. Only applies when the input reports 96 kHz mono.",
        "Les cartes MacroSilicon bon marché étiquettent mal leur audio. S'applique seulement quand l'entrée annonce du 96 kHz mono.",
    ),
    ("Remove DC offset", "Supprimer le décalage DC"),
    (
        "Some cards add a constant offset in their analog audio stage.",
        "Certaines cartes ajoutent un décalage constant dans leur étage audio analogique.",
    ),
    ("Hiss reduction (+11 ms)", "Réduction du souffle (+11 ms)"),
    (
        "Learns the card's hiss while nothing plays, then pulls it down under game audio (spectral noise reduction). Adds ~11 ms of audio latency.",
        "Apprend le souffle de la carte quand rien ne joue, puis le réduit sous le son du jeu (réduction spectrale du bruit). Ajoute ~11 ms de latence audio.",
    ),
    ("learned", "appris"),
    ("learning", "apprentissage"),
    ("Strength", "Force"),
    ("Noise gate", "Noise gate"),
    (
        "Fades the card's hiss to silence when nothing is playing. 1 ms attack, so sounds aren't clipped.",
        "Fait taire le souffle de la carte quand rien ne joue. Attaque de 1 ms : les sons ne sont pas coupés.",
    ),
    ("open", "ouvert"),
    ("closed", "fermé"),
    ("Gate threshold", "Seuil du gate"),
    ("Noise floor", "Bruit de fond"),
    ("set gate", "régler le gate"),
    (
        "Threshold = measured floor + 8 dB. Measure while nothing is playing.",
        "Seuil = bruit mesuré + 8 dB. Mesure quand rien ne joue.",
    ),
    ("measuring...", "mesure…"),
    ("restart audio", "relancer l'audio"),
    ("No matching audio input - pick one above.", "Aucune entrée audio correspondante : choisis-en une plus haut."),
    // --- overlay & capture
    ("Overlay & capture", "Affichage et capture"),
    ("Show stats HUD (F2)", "Afficher le HUD des stats (F2)"),
    ("HUD corner", "Coin du HUD"),
    ("Top left", "Haut gauche"),
    ("Top right", "Haut droite"),
    ("Bottom left", "Bas gauche"),
    ("Bottom right", "Bas droite"),
    ("HUD preset", "Preset du HUD"),
    ("FPS", "FPS"),
    ("Pacing", "Cadence"),
    ("Latency", "Latence"),
    ("Upscale", "Upscale"),
    ("Header", "En-tête"),
    ("Boxed", "Encadré"),
    ("HUD opacity", "Opacité du HUD"),
    ("snap", "coin"),
    ("Back to the corner (or double-click the HUD)", "Retour au coin (ou double-clic sur le HUD)"),
    ("Drag the HUD with the mouse to place it anywhere.", "Glisse le HUD à la souris pour le placer où tu veux."),
    ("Language", "Langue"),
    ("UI scale", "Échelle de l'interface"),
    ("screenshot", "capture d'écran"),
    ("Ctrl+S / F12", "Ctrl+S / F12"),
    ("open folder", "ouvrir le dossier"),
    // --- HUD lines
    ("SIGNAL", "SIGNAL"),
    ("WARN", "ALERTE"),
    ("GPU/CPU", "GPU/CPU"),
    // --- no signal card
    ("No signal", "Pas de signal"),
    ("Waiting for an HDMI source", "En attente d'une source HDMI"),
    ("Opening device...", "Ouverture de la carte…"),
    ("No capture devices found", "Aucune carte de capture trouvée"),
    ("F1 · tab · right-click — settings", "F1 · tab · clic droit — réglages"),
    // --- toasts
    ("Ghost mode - Ctrl+Alt+G to leave", "Mode fantôme — Ctrl+Alt+G pour quitter"),
    ("Ghost mode off", "Mode fantôme désactivé"),
    ("Preset not found (renamed or deleted?)", "Preset introuvable (renommé ou supprimé ?)"),
    ("Deleted preset", "Preset supprimé"),
    ("Picture reset", "Image réinitialisée"),
    ("Muted", "Son coupé"),
    ("Unmuted", "Son rétabli"),
    ("Pinned on top", "Épinglé au premier plan"),
    ("Unpinned", "Désépinglé"),
    ("Integer", "Échelle entière"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_ignores_punctuation_and_falls_back() {
        set(true);
        assert_eq!(tr("Loading frame\u{2026}"), "Chargement du cadre…");
        assert_eq!(tr("Opening device\u{2026}"), "Ouverture de la carte…");
        assert_eq!(tr("not a known string"), "not a known string");
        assert_eq!(tr_dyn("Saved preset Foo"), "Preset enregistré : Foo");
        assert_eq!(tr_dyn("Exported 3 profiles"), "Exporté : 3 profils");
        set(false);
        assert_eq!(tr("Brightness"), "Brightness");
        set(true);
    }

    #[test]
    fn no_two_entries_collide_with_different_text() {
        let mut seen: HashMap<String, &str> = HashMap::new();
        for (en, fr) in TABLE {
            if let Some(prev) = seen.insert(norm(en), fr) {
                assert_eq!(prev, *fr, "conflicting translations for {en}");
            }
        }
    }
}
