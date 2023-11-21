module.exports = {
  mode: "jit",
  content: {
    files: ["src/**/*.rs", "index.html"],
  },
  darkMode: "media", // 'media' or 'class'
  theme: {
    extend: {},
  },
  variants: {
    extend: {},
  },
  plugins: [],
  safelist: [ // Can't figure out all the colours, so we force it
      "border-red-900", "bg-red-900", "text-red-900", "accent-red-900",
      "border-lime-900", "bg-lime-900", "text-lime-900", "accent-lime-900",
      "border-cyan-900", "bg-cyan-900", "text-cyan-900", "accent-cyan-900",
      "border-violet-900", "bg-violet-900", "text-violet-900", "accent-violet-900",
      "border-orange-900", "bg-orange-900", "text-orange-900", "accent-orange-900",
      "border-green-900", "bg-green-900", "text-green-900", "accent-green-900",
      "border-sky-900", "bg-sky-900", "text-sky-900", "accent-sky-900",
      "border-purple-900", "bg-purple-900", "text-purple-900", "accent-purple-900",
      "border-amber-900", "bg-amber-900", "text-amber-900", "accent-amber-900",
      "border-emerald-900", "bg-emerald-900", "text-emerald-900", "accent-emerald-900",
      "border-blue-900", "bg-blue-900", "text-blue-900", "accent-blue-900",
      "border-fuchsia-900", "bg-fuchsia-900", "text-fuchsia-900", "accent-fuchsia-900",
      "border-yellow-900", "bg-yellow-900", "text-yellow-900", "accent-yellow-900",
      "border-teal-900", "bg-teal-900", "text-teal-900", "accent-teal-900",
      "border-indigo-900", "bg-indigo-900", "text-indigo-900", "accent-indigo-900",
      "border-pink-900", "bg-pink-900", "text-pink-900", "accent-pink-900",
      "border-rose-900", "bg-rose-900", "text-rose-900", "accent-rose-900",
  ],
};
