export default {
  logo: <h1 className="font-bold text-4xl md:text-4xl lg:text-5xl">Oura</h1>,
  project: {
    link: "https://github.com/txpipe/oura",
  },
  chat: {
    link: "https://discord.gg/Vc3x8N9nz2",
  },
  footer: {
    text: "Oura - TxPipe",
  },
  nextThemes: {
    defaultTheme: "dark",
  },
  docsRepositoryBase: "https://github.com/txpipe/oura/tree/main/docs",
  useNextSeoProps() {
    return {
      titleTemplate: "%s – Oura",
      siteName: "Oura",
    };
  },
};
