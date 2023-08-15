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
      description: "The tail of Cardano",
      canonical: "https://oura.txpipe.io",
      siteName: "Oura",
      openGraph: {
        url: "https://oura.txpipe.io",
        title: "Oura",
        description: "The tail of Cardano",
        images: [
          {
            url: "https://oura.txpipe.io/logo.svg",
            width: 732,
            height: 287,
            alt: "Oura",
            type: "image/svg",
          },
        ],
      },
    };
  },
};
