const root = document.getElementById('root');

const unknownErrorOrReason = 'unknown error or reason';
const someErrorOccurred = 'some error occurred';

const ruleInputID = 'rule-input';
const logsOutputID = 'logs-output';

const paragraph = 'p';

const newLogClass = 'new-log';
const errorClass = 'error';
const logClass = 'log';

const maxElements = 25;
let elCount = 0, loEls = [];

const newLogTimeout = 500;
const errorTimeout = 5000;

let ws = null;

const stop = () => {
  if (ws !== null) {
    ws.onclose = () => {
      // disable onclose handler first
    };
    ws.close();
    ws = null;
  }
};

const start = () => {
  stop();

  const riEl = document.getElementById(ruleInputID);
  const rule = riEl.value;

  ws = new WebSocket(`ws://127.0.0.1:${wsPort}/?rule=${rule}`);
  initWebsocket();
};

window.onbeforeunload = () => {
  stop();
};

const error = (error) => {
  if (error === '') error = unknownErrorOrReason;

  const el = document.createElement(paragraph);
  el.id =
    el.innerHTML = error;
  el.classList.add(errorClass);
  root.appendChild(el);

  setTimeout(() => {
    el.remove();
  }, errorTimeout);
};

const initWebsocket = () => {
  ws.onclose = (e) => {
    error(e.reason);
  };

  const loEl = document.getElementById(logsOutputID);

  ws.onmessage = (e) => {
    const el = document.createElement(paragraph);
    el.classList.add(logClass);
    el.classList.add(newLogClass);
    el.innerHTML = e.data;
    loEl.prepend(el);

    elCount++;
    loEls.unshift(el);

    setTimeout(() => {
      el.classList.remove(newLogClass);
    }, newLogTimeout);

    if (elCount === maxElements) {
      elCount--;
      loEls.pop().remove();
    }
  };

  ws.onerror = (error) => {
    error(someErrorOccurred);
  };
};
