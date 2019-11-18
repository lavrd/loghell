// root dashboard element
const root = document.getElementById("root");

// message about unknown error
const unknownErrorOrReason = "unknown error";

// id of input element for rule
const ruleInputID = "rule-input";
// id of input element for logs output
const logsOutputID = "logs-output";

// paragraph html element tag
const paragraph = "p";

// new log element class name
const newLogClass = "new-log";
// error element class name
const errorClass = "error";
// old logs elements class name
const logClass = "log";

// mac logs on screen
const maxElements = 25;
// current elements count and array for this elements
let elCount = 0, loEls = [];

// in this timeout log is considered as a new
const newLogTimeout = 500;
// in this timeout error is on the screen
const errorTimeout = 5000;

// websocket variable
let ws = null;

// stop websocket function
const stop = () => {
  if (ws !== null) {
    ws.onclose = () => {
      /* disable onclose handler first */
    };
    ws.close();
    ws = null;
  }
};

// connect to websocket and start to receive logs
const start = () => {
  // first stop old websocket connection
  stop();

  // get rule from inout
  const riEl = document.getElementById(ruleInputID);
  const rule = riEl.value;

  // start new websocket connection
  ws = new WebSocket(`ws://127.0.0.1:${wsPort}/?rule=${rule}`);
  initWebsocket();
};

// execute when user leave from page
window.onbeforeunload = () => {
  stop();
};

// this func display error on the screen
const error = (error) => {
  // if we don't know error message -> display unknown
  if (error === "") error = unknownErrorOrReason;

  // else create an element and display error message
  const el = document.createElement(paragraph);
  el.id = el.innerHTML = error;
  el.classList.add(errorClass);
  root.appendChild(el);

  // also set timeout to remove error
  setTimeout(() => {
    el.remove();
  }, errorTimeout);
};

// init websocket callbacks
const initWebsocket = () => {
  ws.onclose = (e) => {
    error(e.reason);
  };

  const loEl = document.getElementById(logsOutputID);

  // on every message from server
  ws.onmessage = (e) => {
    // create element
    const el = document.createElement(paragraph);
    // add classes
    el.classList.add(logClass);
    el.classList.add(newLogClass);
    // add log
    el.innerHTML = e.data;
    loEl.prepend(el);

    // increase logs count
    elCount++;
    // and add to elements array
    loEls.unshift(el);

    // set timeout for remote new log class
    setTimeout(() => {
      el.classList.remove(newLogClass);
    }, newLogTimeout);

    // and if elements in array > then max -> delete oldest
    if (elCount === maxElements) {
      elCount--;
      loEls.pop().remove();
    }
  };

  ws.onerror = (error) => {
    error(unknownErrorOrReason);
  };
};
