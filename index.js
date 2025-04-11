// ADD THE <script src="./target/wasm32-unknown-emscripten/release/nbs-player-rs.js"> TAG TO THE PAGE
const scriptTag = document.createElement("script");
scriptTag.src = "./target/wasm32-unknown-emscripten/release/nbs-player-rs.js";

// Append the script tag to the body
document.body.appendChild(scriptTag);

const themes = {
  BLOOD_MOON: {
    background_color: "#2E1A1A", // Dark red-brown
    accent_color: "#FF0000", // Blood red
    text_color: "#E0E0E0", // Light gray
    white_key_color: "#444444", // Dark gray
    black_key_color: "#1A1A1A", // Near black
    white_text_key_color: "#E0E0E0", // Light gray
    black_text_key_color: "#FF0000", // Blood red
  },
  ANCIENT_SCROLL: {
    background_color: "#6B4423", // Brown (like wood)
    accent_color: "#8B4513", // Saddle brown
    text_color: "#FAFAD2", // Light goldenrod
    white_key_color: "#D2B48C", // Tan (like parchment)
    black_key_color: "#3B2F2F", // Dark brown
    white_text_key_color: "#1A1A1A", // Dark gray
    black_text_key_color: "#FAFAD2", // Light goldenrod
  },
  NEON_GRID: {
    background_color: "#0A0A1E", // Deep space blue
    accent_color: "#00FF00", // Neon green
    text_color: "#FFFFFF", // Pure white
    white_key_color: "#1E1E1E", // Dark gray
    black_key_color: "#000000", // Pure black
    white_text_key_color: "#00FF00", // Neon green
    black_text_key_color: "#FFFFFF", // Pure white
  },
  SYNTHWAVE_DREAM: {
    background_color: "#1A1A2E", // Deep purple-blue
    accent_color: "#FF007F", // Bright pink
    text_color: "#00FFFF", // Cyan
    white_key_color: "#333333", // Dark gray
    black_key_color: "#000000", // Pure black
    white_text_key_color: "#00FFFF", // Cyan
    black_text_key_color: "#FF007F", // Bright pink
  },
  GOLDEN_PARCHMENT: {
    background_color: "#8B7355", // Burnt sienna
    accent_color: "#DAA520", // Goldenrod
    text_color: "#FFF8DC", // Cornsilk
    white_key_color: "#F5F5DC", // Beige
    black_key_color: "#654321", // Dark brown
    white_text_key_color: "#1A1A1A", // Dark gray
    black_text_key_color: "#FFF8DC", // Cornsilk
  },
  COTTON_CANDY: {
    background_color: "#87CEEB", // Sky blue
    accent_color: "#FF69B4", // Hot pink
    text_color: "#FFFFFF", // Pure white
    white_key_color: "#E6E6FA", // Lavender
    black_key_color: "#9370DB", // Medium purple
    white_text_key_color: "#1A1A1A", // Dark gray
    black_text_key_color: "#FFFFFF", // Pure white
  },
  MIDNIGHT_JAZZ: {
    background_color: "#121212", // Near-black
    accent_color: "#6A5ACD", // Slate blue
    text_color: "#E0E0E0", // Light gray
    white_key_color: "#333333", // Dark gray
    black_key_color: "#1A1A1A", // Deeper black
    white_text_key_color: "#E0E0E0",
    black_text_key_color: "#6A5ACD", // Slate blue
  },
  CYBERPUNK_SUNSET: {
    background_color: "#1A1A2E", // Dark blue
    accent_color: "#FF6B35", // Neon orange
    text_color: "#00F5D4", // Teal
    white_key_color: "#29293D",
    black_key_color: "#00001A",
    white_text_key_color: "#00F5D4",
    black_text_key_color: "#FF6B35",
  },
  MATCHA_LATTE: {
    background_color: "#E8F5E9", // Light mint
    accent_color: "#4CAF50", // Matcha green
    text_color: "#3E2723", // Dark brown
    white_key_color: "#F1F8E9",
    black_key_color: "#C8E6C9",
    white_text_key_color: "#3E2723",
    black_text_key_color: "#4CAF50",
  },
  OBSIDIAN_GLOW: {
    background_color: "#000000",
    accent_color: "#00B4D8", // Electric blue
    text_color: "#FFFFFF",
    white_key_color: "#1E1E1E",
    black_key_color: "#000000",
    white_text_key_color: "#00B4D8",
    black_text_key_color: "#FFFFFF",
  },
  BLUE_SKY: {
    background_color: "#87CEEB", // Light blue
    accent_color: "#4682B4", // Steel blue
    text_color: "#FFFFFF",
    white_key_color: "#F0F8FF",
    black_key_color: "#B0C4DE",
    white_text_key_color: "#4682B4",
    black_text_key_color: "#FFFFFF",
  },
  ROSE_SUNSET: {
    background_color: "#FFE4E1", // Soft pink
    accent_color: "#D4A59A", // Rosé
    text_color: "#5D4037", // Dark brown
    white_key_color: "#FFF0F5",
    black_key_color: "#E6C7C2",
    white_text_key_color: "#5D4037",
    black_text_key_color: "#D4A59A",
  },
  DARK_FOREST: {
    background_color: "#1E3F20", // Forest green
    accent_color: "#8F9779", // Sage
    text_color: "#E0E0E0",
    white_key_color: "#3B5323",
    black_key_color: "#1A1F16",
    white_text_key_color: "#E0E0E0",
    black_text_key_color: "#8F9779",
  },
  RETRO_VHS: {
    background_color: "#0D0221", // Deep purple-black
    accent_color: "#FF2A6D", // VHS pink
    text_color: "#05D9E8", // VHS cyan
    white_key_color: "#1A1A2E",
    black_key_color: "#000000",
    white_text_key_color: "#05D9E8",
    black_text_key_color: "#FF2A6D",
  },
  SAKURA_GARDEN: {
    background_color: "#FFF0F5", // Blush white
    accent_color: "#FF85A2", // Sakura pink
    text_color: "#5D4037", // Dark brown
    white_key_color: "#FFEBEE",
    black_key_color: "#F8BBD0",
    white_text_key_color: "#5D4037",
    black_text_key_color: "#FF85A2",
  },
  LAVA_LAMP: {
    background_color: "#2D0B4F", // Deep purple
    accent_color: "#FF6E00", // Lava orange
    text_color: "#FFFFFF",
    white_key_color: "#4B1E88",
    black_key_color: "#1A0033",
    white_text_key_color: "#FF6E00",
    black_text_key_color: "#FFFFFF",
  },
};

const songs = [
  {
    filename: "Espresso.zip",
    title: "Sabrina Carpenter - Espresso",
    description: "https://noteblock.world/song/zya0Bgz4rd",
    theme: themes.ROSE_SUNSET,
    fontId: "PixelPlay",
    background: "Plasma",
  },
  {
    filename: "360.zip",
    title: "Charli XCX - 360",
    description:
      "Charli XCX - 360 is a song by Charli XCX, an English singer and songwriter. The song features a catchy pop melody and upbeat production.\n\nhttps://noteblock.world/song/0N1vDGTtSF",
    theme: themes.SAKURA_GARDEN,
    fontId: "Romulus",
    background: "Water",
  },
  {
    filename: "bo en - My Time.zip",
    title: "bo en - My Time",
    description: `Popular song by bo en featured in OMORI.\n\nhttps://noteblock.world/song/LtZaBBTBfS`,
    theme: themes.BLUE_SKY,
    fontId: "PixAntiqua",
    background: "Water",
  },
  {
    filename: "Bad Piggies Theme.nbs",
    title: "Ilmari Hakkola - Bad Piggies Theme",
    description: `The "Bad Piggies Theme" is the main theme of the 2012 video game Bad Piggies, a spinoff of the Angry Birds series. The game features the Bad Piggies as they attempt to steal the Bird's eggs.\n\nhttps://noteblock.world/song/0N1vDGTtSF`,
    theme: themes.MATCHA_LATTE,
    fontId: "Monocraft",
    background: "Grass",
  },
  {
    filename: "Rush E.nbs",
    title: "mikamohr2506 - Rush E",
    description: `https://noteblock.world/song/tA0KJdJC5h`,
    theme: themes.GOLDEN_PARCHMENT,
    fontId: "PixAntiqua",
    background: "WaterFall",
  },
  {
    filename: "turkish_march.nbs",
    title: "Turkish March",
    description: "Mozart's famous piano piece, Rondo Alla Turca",
    theme: themes.MIDNIGHT_JAZZ,
    fontId: "PixAntiqua",
    background: "Grass",
  },
  {
    filename: "Darude - Sandstorm.zip",
    title: "Darude - Sandstorm",
    description: "https://noteblock.world/song/zEdaMYZmX0",
    theme: themes.NEON_GRID,
    fontId: "Setbackt",
    background: "Sand",
  },

  {
    filename: "Ievan Polkka - Hatsune Miku.nbs",
    title: "Ievan Polkka - Hatsune Miku",
    description:
      "Finnish folk song popularized by Hatsune Miku\n\nhttps://noteblock.world/song/3t6Sk2kdMA",
    theme: themes.OBSIDIAN_GLOW,
    fontId: "PixAntiqua",
    background: "Voronoise",
  },
];

// Store current song index in localStorage
let currentSongIndex = localStorage.getItem("currentSongIndex");
if (currentSongIndex === null) {
  currentSongIndex = 0;
} else {
  currentSongIndex = parseInt(currentSongIndex);
  if (currentSongIndex >= songs.length) {
    currentSongIndex = 0;
  }
}

const songInfoElement = document.getElementById("songInfo");
const songDescElement = document.getElementById("songDesc");
const prevBtn = document.getElementById("prevBtn");
const nextBtn = document.getElementById("nextBtn");
const playlistElement = document.getElementById("playlist");

function updateUI() {
  const currentSong = songs[currentSongIndex];

  // Update song info
  songInfoElement.textContent = `Now Playing: ${currentSong.title}`;
  songDescElement.textContent = currentSong.description;

  // Update playlist display
  playlistElement.innerHTML = "";
  songs.forEach((song, index) => {
    const item = document.createElement("div");
    item.className = `playlist-item ${
      index === currentSongIndex ? "current" : ""
    }`;
    item.textContent = song.title;
    item.addEventListener("click", () => {
      loadSong(index);
    });
    playlistElement.appendChild(item);
  });
}

function loadSong(index) {
  currentSongIndex = index;
  localStorage.setItem("currentSongIndex", currentSongIndex);

  // Get the selected song's settings
  const selectedSong = songs[currentSongIndex];

  // Update the arguments with the song's theme and font
  arguments.theme = selectedSong.theme;
  arguments.font_id = selectedSong.fontId;

  // Reload the page to play the new song
  location.reload();
}

function prevSong() {
  currentSongIndex = (currentSongIndex - 1 + songs.length) % songs.length;
  loadSong(currentSongIndex);
}

function nextSong() {
  currentSongIndex = (currentSongIndex + 1) % songs.length;
  loadSong(currentSongIndex);
}

prevBtn.addEventListener("click", prevSong);
nextBtn.addEventListener("click", nextSong);

let fullscreen_window_width = window.screen.width;
let fullscreen_window_height = window.screen.height;
// if window width is less than height, swap them
if (fullscreen_window_width < fullscreen_window_height) {
  const temp = fullscreen_window_width;
  fullscreen_window_width = fullscreen_window_height;
  fullscreen_window_height = temp;
}

fullscreen_window_width = Math.floor(fullscreen_window_width * 0.75);
fullscreen_window_height = Math.floor(fullscreen_window_height * 0.75);

// 720p resolution
const window_width = 1280;
const window_height = 720;

// Initialize arguments with the current song's theme and font
const arguments = {
  font_id: songs[currentSongIndex].fontId,
  background: songs[currentSongIndex].background,
  window_width: fullscreen_window_width,
  window_height: fullscreen_window_height,
  theme: songs[currentSongIndex].theme,
};

const canvas = document.getElementById("canvas");
// if spacebar is pressed do not roll the screen
canvas.addEventListener("keydown", function (event) {
  if (event.code === "Space") {
    event.preventDefault();
  }
});

var Module = {
  canvas: canvas,
  arguments: [JSON.stringify(arguments)],
  noInitialRun: true,
  noExitRuntime: false, // Allow the runtime to exit
  preInit: async function () {
    updateUI();

    // wait 2 seconds before starting
    await new Promise((resolve) => setTimeout(resolve, 200));
    const song_url = `./test-assets/${songs[currentSongIndex].filename}`;
    const response = await fetch(song_url);
    const arrayBuffer = await response.arrayBuffer();
    const byteArray = new Uint8Array(arrayBuffer);
    FS.writeFile("/song.nbsx", byteArray);
    callMain(Module.arguments);
  },
};

// Add keyboard shortcuts
document.addEventListener("keydown", (e) => {
  if (e.code === "ArrowLeft") {
    prevSong();
  } else if (e.code === "ArrowRight") {
    nextSong();
  }
});
