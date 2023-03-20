// global state
let counter = 0;

globalThis.mapEvent = function (record) {
  // I can store custom state that will be available on the next iteration
  counter += 1;

  return {
    msg: "I can change this at runtime without having to compile Oura!",
    counter,
  };
};
