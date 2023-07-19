export function Hero() {
  return (
    <div className="md:p-20">
      <div className="mb-5 text-center">
        <h1 className="flex flex-col flex-wrap font-bold text-4xl md:text-6xl lg:text-7xl dark:text-gray-200">
          <img src="/logo.svg" alt="Oura" className="mb-4" />
          The tail of Cardano
        </h1>
      </div>

      <div className="mb-8 max-w-3xl text-center mx-auto">
        <p className="text-lg text-gray-600 dark:text-gray-400">
          Oura is a rust-native implementation of a pipeline that connects to
          the tip of a Cardano node through a combination of Ouroboros
          mini-protocol (using either a unix socket or tcp bearer), filters the
          events that match a particular pattern and then submits a succinct,
          self-contained payload to pluggable observers called "sinks".
        </p>
      </div>

      <div className="flex justify-center">
        <a
          className="bg-gradient-to-tl from-emerald-600 to-cyan-600 border border-transparent text-white text-sm rounded-md focus:outline-none focus:ring-2 focus:ring-blue-600 focus:ring-offset-2 focus:ring-offset-white py-3 px-4 dark:focus:ring-offset-gray-800"
          href="/v2"
        >
          Documentation
        </a>
      </div>
    </div>
  );
}
