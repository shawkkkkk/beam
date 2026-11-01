export type Token = {
  id: string;
  name: string;
  symbol: string;
  description: string;
  icon: string;
  colors: [string, string];
  marketCap: string;
  volume: string;
  holders: number;
  age: string;
  progress: number;
  change: number;
  isDraft?: boolean;
};

export const sampleTokens: Token[] = [
  {
    id: "nova",
    name: "Nova Club",
    symbol: "NOVA",
    description: "A little spark for the internet after dark.",
    icon: "✳",
    colors: ["#5a46ed", "#9d79ff"],
    marketCap: "$84.2K",
    volume: "$18.4K",
    holders: 124,
    age: "2m",
    progress: 87,
    change: 28.6,
  },
  {
    id: "orbit",
    name: "Orbit",
    symbol: "ORBIT",
    description: "Good things come back around.",
    icon: "◎",
    colors: ["#1967e7", "#70c4ff"],
    marketCap: "$62.8K",
    volume: "$12.1K",
    holders: 98,
    age: "6m",
    progress: 72,
    change: 16.2,
  },
  {
    id: "pixel",
    name: "Pixel Ghost",
    symbol: "GHOST",
    description: "Friendly haunting, on chain.",
    icon: "◈",
    colors: ["#6b54ec", "#3d75e8"],
    marketCap: "$51.3K",
    volume: "$9.7K",
    holders: 82,
    age: "11m",
    progress: 59,
    change: 12.8,
  },
  {
    id: "wave",
    name: "Waveform",
    symbol: "WAVE",
    description: "Catch the signal. Ride the wave.",
    icon: "≈",
    colors: ["#2873e8", "#8c53e8"],
    marketCap: "$38.6K",
    volume: "$7.3K",
    holders: 64,
    age: "18m",
    progress: 46,
    change: -4.1,
  },
  {
    id: "lumen",
    name: "Lumen",
    symbol: "LUMEN",
    description: "A brighter kind of internet magic.",
    icon: "✦",
    colors: ["#7354f8", "#b876ff"],
    marketCap: "$29.5K",
    volume: "$5.4K",
    holders: 51,
    age: "32m",
    progress: 34,
    change: 8.7,
  },
  {
    id: "echo",
    name: "Echo Room",
    symbol: "ECHO",
    description: "Make some noise that lasts.",
    icon: "◉",
    colors: ["#2464d4", "#a75cf7"],
    marketCap: "$21.8K",
    volume: "$4.1K",
    holders: 37,
    age: "1h",
    progress: 26,
    change: 5.4,
  },
  {
    id: "drift",
    name: "Drift Club",
    symbol: "DRIFT",
    description: "For everyone going somewhere new.",
    icon: "↗",
    colors: ["#3f74ed", "#6f4cdc"],
    marketCap: "$13.2K",
    volume: "$2.8K",
    holders: 28,
    age: "2h",
    progress: 16,
    change: -2.3,
  },
  {
    id: "arc",
    name: "Arcade",
    symbol: "ARCADE",
    description: "One more round. One more chance.",
    icon: "⌁",
    colors: ["#8d4cf2", "#3369f0"],
    marketCap: "$9.4K",
    volume: "$1.6K",
    holders: 19,
    age: "3h",
    progress: 11,
    change: 3.8,
  },
];
