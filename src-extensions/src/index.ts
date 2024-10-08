import { createConnection } from "node:net";
import { ExtensionHostIPCHandler } from "./sandbox";
import { EventEmitter } from "node:events";
import { Mutex } from "async-mutex";

const IPC_PATH =
  process.argv[process.argv.findIndex((val) => val === "-ipcPath") + 1];

console.log("got IPC path", process.argv);

const client = createConnection(IPC_PATH);

const mutex = new Mutex();

client.on("connect", (err) => {
  if (err) {
    console.error(err);
    return;
  }

  const bus = new EventEmitter();
  const extHandler = new ExtensionHostIPCHandler(bus);

  const channelMap = {};
  bus.on("request", (data) => {
    const channel = data?.channel;
    if (channel) {
      channelMap[channel] = true;

      mutex.runExclusive(() => {
        client.write(`${JSON.stringify(data)}\n`);
      });
    }
  });

  let oldBuffer: Buffer;

  client.on("data", async (data) => {
    let newData = Buffer.concat([oldBuffer ?? Buffer.alloc(0), data]);

    while (true) {
      const index = newData.findIndex((val) => val === "\n".charCodeAt(0));
      if (index === -1) {
        oldBuffer = newData;
        return;
      }

      const line = newData.subarray(0, index).toString();
      newData = newData.subarray(index + 1, newData.length);
      try {
        const parsed = JSON.parse(line.toString().trim());
        const channel = parsed?.channel;
        if (channel && channelMap[channel]) {
          bus.emit(channel, parsed);
          continue;
        }

        const dataRet = await extHandler.parseMessage(parsed);

        const ret: mainHostMessage = {
          type: parsed.type,
          data: dataRet,
          channel: parsed.channel,
          extensionName: parsed.extensionName,
        };

        mutex.runExclusive(() => {
          client.write(`${JSON.stringify(ret)}\n`);
        });
      } catch (e) {
        console.error(e);
      }
    }
  });
});
