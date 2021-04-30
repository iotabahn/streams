const streams = require("../wasm-node/iota_streams_wasm");
const fetch = require("node-fetch");

global.fetch = fetch;
global.Headers = fetch.Headers;
global.Request = fetch.Request;
global.Response = fetch.Response;

streams.set_panic_hook();

main()
  .then(() => {
    console.log("Done example");
  })
  .catch((err) => {
    console.log(err);
  });

async function main() {
  let node = "https://api.lb-0.testnet.chrysalis2.com/";
  let options = new streams.SendOptions(9, true, 1);
  let seed = make_seed(81);
  let auth = new streams.Author(node, seed, options, false);

  console.log("channel address: ", auth.channel_address());
  console.log("multi branching: ", auth.is_multi_branching());

  let response = await auth.clone().send_announce();
  let ann_link = response.get_link();
  console.log("announced at: ", ann_link.to_string());

  let options2 = new streams.SendOptions(9, true, 1);
  let seed2 = make_seed(81);
  let sub = new streams.Subscriber(node, seed2, options2);
  let ann_link_copy = ann_link.copy();
  await sub.clone().receive_announcement(ann_link_copy);

  console.log("Subscribing...");
  ann_link_copy = ann_link.copy();
  response = await sub.clone().send_subscribe(ann_link_copy);
  let sub_link = response.get_link();
  console.log("Subscription message at: ", sub_link.to_string());
  await auth.clone().receive_subscribe(sub_link);
  console.log("Subscription processed");

  console.log("Sending Keyload");
  response = await auth.clone().send_keyload_for_everyone(ann_link);
  let keyload_link = response.get_link();
  console.log("Keyload message at: ", keyload_link.to_string());

  console.log("Subscriber syncing...");
  await sub.clone().sync_state();

  let public_payload = to_bytes("Public");
  let masked_payload = to_bytes("Masked");

  console.log("Subscriber Sending tagged packet");
  response = await sub
    .clone()
    .send_tagged_packet(keyload_link, public_payload, masked_payload);
  let tag_link = response.get_link();
  console.log("Tag packet at: ", tag_link.to_string());

  let last_link = tag_link;
  console.log("Subscriber Sending multiple signed packets");

  for (var x = 0; x < 10; x++) {
    response = await sub
      .clone()
      .send_signed_packet(last_link, public_payload, masked_payload);
    last_link = response.get_link();
    console.log("Signed packet at: ", last_link.to_string());
  }

  console.log("\nAuthor fetching next messages");
  let exists = true;
  while (exists) {
    let next_msgs = await auth.clone().fetch_next_msgs();

    if (next_msgs.length === 0) {
      exists = false;
    }

    for (var i = 0; i < next_msgs.length; i++) {
      console.log("Found a message...");
      console.log(
        "Public: ",
        from_bytes(next_msgs[i].get_message().get_public_payload()),
        "\tMasked: ",
        from_bytes(next_msgs[i].get_message().get_masked_payload())
      );
    }
  }

  function to_bytes(str) {
    var bytes = [];
    for (var i = 0; i < str.length; ++i) {
      bytes.push(str.charCodeAt(i));
    }
    return bytes;
  }

  function from_bytes(bytes) {
    var str = "";
    for (var i = 0; i < bytes.length; ++i) {
      str += String.fromCharCode(bytes[i]);
    }
    return str;
  }

  function make_seed(size) {
    const alphabet = "abcdefghijklmnopqrstuvwxyz";
    let seed = "";
    for (i = 9; i < size; i++) {
      seed += alphabet[Math.floor(Math.random() * alphabet.length)];
    }
    return seed;
  }
}
