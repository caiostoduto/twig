use std::fmt;

pub enum MinecraftServerType {
    Lobby = 0,
    Game = 1,
}

pub struct MinecraftMusicTrack {
    pub title: &'static str,
    pub author: &'static str,
    pub duration_secs: &'static u64,
}

impl MinecraftMusicTrack {
    fn new(title: &'static str, author: &'static str, duration_secs: &'static u64) -> Self {
        Self {
            title,
            author,
            duration_secs,
        }
    }
}

impl fmt::Display for MinecraftMusicTrack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} • {}", self.title, self.author)
    }
}

/// Returns a list of Minecraft music tracks by C418
pub fn get_tracks() -> Vec<MinecraftMusicTrack> {
    vec![
        MinecraftMusicTrack::new("Key", "C418", &65),
        MinecraftMusicTrack::new("Door", "C418", &111),
        MinecraftMusicTrack::new("Subwoofer Lullaby", "C418", &208),
        MinecraftMusicTrack::new("Death", "C418", &42),
        MinecraftMusicTrack::new("Living Mice", "C418", &177),
        MinecraftMusicTrack::new("Moog City", "C418", &160),
        MinecraftMusicTrack::new("Haggstrom", "C418", &204),
        MinecraftMusicTrack::new("Minecraft", "C418", &254),
        MinecraftMusicTrack::new("Oxygène", "C418", &65),
        MinecraftMusicTrack::new("Équinoxe", "C418", &114),
        MinecraftMusicTrack::new("Mice on Venus", "C418", &281),
        MinecraftMusicTrack::new("Dry Hands", "C418", &68),
        MinecraftMusicTrack::new("Wet Hands", "C418", &90),
        MinecraftMusicTrack::new("Clark", "C418", &191),
        MinecraftMusicTrack::new("Chris", "C418", &87),
        MinecraftMusicTrack::new("Thirteen", "C418", &176),
        MinecraftMusicTrack::new("Excuse", "C418", &124),
        MinecraftMusicTrack::new("Sweden", "C418", &215),
        MinecraftMusicTrack::new("Cat", "C418", &186),
        MinecraftMusicTrack::new("Dog", "C418", &145),
        MinecraftMusicTrack::new("Danny", "C418", &254),
        MinecraftMusicTrack::new("Beginning", "C418", &102),
        MinecraftMusicTrack::new("Droopy Likes Ricochet", "C418", &96),
        MinecraftMusicTrack::new("Droopy Likes Your Face", "C418", &117),
    ]
}
