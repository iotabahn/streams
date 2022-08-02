use futures::executor::block_on;
use wasm_bindgen::prelude::*;

use crate::{
    types::{
        PskIds as PskIdsW,
        PublicKeys as PublicKeysW,
        *,
    },
    user::userw::*,
};
use js_sys::Array;

use core::cell::RefCell;

/// Streams imports
use iota_streams::{
    app::{
        identifier::Identifier,
        transport::{
            tangle::client::Client as ApiClient,
            TransportOptions,
        },
    },
    app_channels::api::{
        psk_from_seed,
        pskid_from_psk,
        tangle::{
            futures::TryStreamExt,
            Author as ApiAuthor,
        },
    },
    core::{
        prelude::{
            Rc,
            String,
            ToString,
        },
        psk::pskid_from_hex_str,
        psk::pskid_to_hex_string,
    },
    ddml::types::*,
};

#[wasm_bindgen]
pub struct Author {
    // Don't alias away the ugliness, so we don't forget
    author: Rc<RefCell<ApiAuthor<Rc<RefCell<ApiClient>>>>>,
}

#[wasm_bindgen]
impl Author {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: String, options: SendOptions, implementation: ChannelType) -> Author {
        let mut client = ApiClient::new_from_url(&options.url());
        client.set_send_options(options.into());
        let transport = Rc::new(RefCell::new(client));
        let author = Rc::new(RefCell::new(ApiAuthor::new(&seed, implementation.into(), transport)));
        Author { author }
    }

    #[wasm_bindgen(catch, js_name = "fromClient")]
    pub fn from_client(client: StreamsClient, seed: String, implementation: ChannelType) -> Author {
        let author = Rc::new(RefCell::new(ApiAuthor::new(
            &seed,
            implementation.into(),
            client.into_inner(),
        )));
        Author { author }
    }

    #[wasm_bindgen(catch)]
    pub fn import(client: StreamsClient, bytes: Vec<u8>, password: &str) -> Result<Author> {
        block_on(ApiAuthor::import(&bytes, password, client.into_inner()))
            .map(|v| Author {
                author: Rc::new(RefCell::new(v)),
            })
            .into_js_result()
    }

    #[wasm_bindgen(catch)]
    pub fn export(&self, password: &str) -> Result<Vec<u8>> {
        block_on(self.author.borrow_mut().export(password)).into_js_result()
    }

    pub async fn recover(
        seed: String,
        ann_address: Address,
        implementation: ChannelType,
        options: SendOptions,
    ) -> Result<Author> {
        let mut client = ApiClient::new_from_url(&options.url());
        client.set_send_options(options.into());
        let transport = Rc::new(RefCell::new(client));

        ApiAuthor::recover(&seed, ann_address.as_inner(), implementation.into(), transport)
            .await
            .map(|auth| Author {
                author: Rc::new(RefCell::new(auth)),
            })
            .into_js_result()
    }

    pub fn clone(&self) -> Author {
        Author {
            author: self.author.clone(),
        }
    }

    #[wasm_bindgen(catch)]
    pub fn channel_address(&self) -> Result<String> {
        self.author
            .borrow_mut()
            .channel_address()
            .map(|addr| addr.to_string())
            .ok_or("channel not created")
            .into_js_result()
    }

    #[wasm_bindgen(catch, js_name = "announcementLink")]
    pub fn announcement_link(&self) -> Option<String> {
        self.author
            .borrow_mut()
            .announcement_link()
            .map(|addr| addr.to_string())
    }

    #[wasm_bindgen(catch)]
    pub fn is_multi_branching(&self) -> Result<bool> {
        Ok(self.author.borrow_mut().is_multi_branching())
    }

    #[wasm_bindgen(catch)]
    pub fn get_client(&self) -> StreamsClient {
        StreamsClient(self.author.borrow_mut().get_transport().clone())
    }

    #[wasm_bindgen(catch)]
    pub fn store_psk(&self, psk_seed_str: String) -> Result<String> {
        let psk = psk_from_seed(psk_seed_str.as_bytes());
        let pskid = pskid_from_psk(&psk);
        let pskid_str = pskid_to_hex_string(&pskid);
        self.author.borrow_mut().store_psk(pskid, psk).into_js_result()?;
        Ok(pskid_str)
    }

    #[wasm_bindgen(catch)]
    pub fn get_public_key(&self) -> Result<String> {
        Ok(public_key_to_string(self.author.borrow_mut().get_public_key()))
    }

    #[wasm_bindgen(catch)]
    pub async fn send_announce(self) -> Result<UserResponse> {
        self.author
            .borrow_mut()
            .send_announce()
            .await
            .map(|addr| UserResponse::new(addr.into(), None, MessageType::Announce, None))
            .into_js_result()
    }

    #[wasm_bindgen(catch)]
    pub async fn send_keyload_for_everyone(self, link: Address) -> Result<UserResponse> {
        self.author
            .borrow_mut()
            .send_keyload_for_everyone(link.as_inner())
            .await
            .map(|(link, seq_link)| {
                UserResponse::new(link.into(), seq_link.map(Into::into), MessageType::Keyload, None)
            })
            .into_js_result()
    }

    #[wasm_bindgen(catch)]
    pub async fn send_keyload(self, link: Address, psk_ids: PskIdsW, sig_pks: PublicKeysW) -> Result<UserResponse> {
        let pks = sig_pks.pks.into_iter().map(Into::<Identifier>::into);
        let psks = psk_ids.ids.into_iter().map(Into::<Identifier>::into);
        let identifiers: Vec<Identifier> = pks.chain(psks).collect();
        self.author
            .borrow_mut()
            .send_keyload(link.as_inner(), &identifiers)
            .await
            .map(|(link, seq_link)| {
                UserResponse::new(link.into(), seq_link.map(Into::into), MessageType::Keyload, None)
            })
            .into_js_result()
    }

    #[wasm_bindgen(catch)]
    pub async fn send_tagged_packet(
        self,
        link: Address,
        public_payload: Vec<u8>,
        masked_payload: Vec<u8>,
    ) -> Result<UserResponse> {
        self.author
            .borrow_mut()
            .send_tagged_packet(
                link.as_inner(),
                &Bytes(public_payload.clone()),
                &Bytes(masked_payload.clone()),
            )
            .await
            .map(|(link, seq_link)| {
                UserResponse::new(link.into(), seq_link.map(Into::into), MessageType::TaggedPacket, None)
            })
            .into_js_result()
    }

    #[wasm_bindgen(catch)]
    pub async fn send_signed_packet(
        self,
        link: Address,
        public_payload: Vec<u8>,
        masked_payload: Vec<u8>,
    ) -> Result<UserResponse> {
        self.author
            .borrow_mut()
            .send_signed_packet(link.as_inner(), &Bytes(public_payload), &Bytes(masked_payload))
            .await
            .map(|(link, seq_link)| {
                UserResponse::new(link.into(), seq_link.map(Into::into), MessageType::SignedPacket, None)
            })
            .into_js_result()
    }

    #[wasm_bindgen(catch)]
    pub async fn receive_subscribe(self, link_to: Address) -> Result<()> {
        self.author
            .borrow_mut()
            .receive_subscribe(link_to.as_inner())
            .await
            .into_js_result()
    }

    #[wasm_bindgen(catch)]
    pub async fn receive_unsubscribe(self, link_to: Address) -> Result<()> {
        self.author
            .borrow_mut()
            .receive_unsubscribe(link_to.as_inner())
            .await
            .into_js_result()
    }

    #[wasm_bindgen(catch)]
    pub async fn receive_tagged_packet(self, link: Address) -> Result<UserResponse> {
        self.author
            .borrow_mut()
            .receive_tagged_packet(link.as_inner())
            .await
            .map(|(pub_bytes, masked_bytes)| {
                UserResponse::new(
                    link,
                    None,
                    MessageType::TaggedPacket,
                    Some(Message::new(None, pub_bytes.0, masked_bytes.0)),
                )
            })
            .into_js_result()
    }

    #[wasm_bindgen(catch)]
    pub async fn receive_signed_packet(self, link: Address) -> Result<UserResponse> {
        self.author
            .borrow_mut()
            .receive_signed_packet(link.as_inner())
            .await
            .map(|(pk, pub_bytes, masked_bytes)| {
                UserResponse::new(
                    link,
                    None,
                    MessageType::SignedPacket,
                    Some(Message::new(
                        Some(public_key_to_string(&pk)),
                        pub_bytes.0,
                        masked_bytes.0,
                    )),
                )
            })
            .into_js_result()
    }

    #[wasm_bindgen(catch)]
    pub async fn receive_sequence(self, link: Address) -> Result<Address> {
        self.author
            .borrow_mut()
            .receive_sequence(link.as_inner())
            .await
            .map(Into::into)
            .into_js_result()
    }

    pub async fn receive_msg(self, link: Address) -> Result<UserResponse> {
        self.author
            .borrow_mut()
            .receive_msg(link.as_inner())
            .await
            .map(get_message_content)
            .into_js_result()
    }

    pub async fn receive_msg_by_sequence_number(self, anchor_link: Address, msg_num: u32) -> Result<UserResponse> {
        self.author
            .borrow_mut()
            .receive_msg_by_sequence_number(anchor_link.as_inner(), msg_num)
            .await
            .map(get_message_content)
            .into_js_result()
    }

    /// Fetch all the pending messages that the user can read so as to bring the state of the user up to date
    ///
    /// This is the main method to bring the user to the latest state of the channel in order to be able to
    /// publish new messages to it. It makes sure that the messages are processed in topologically order
    /// (ie parent messages before child messages), ensuring a consistent state regardless of the order of publication.
    ///
    /// @returns {number} the amount of messages processed
    /// @throws Throws error if an error has happened during message retrieval.
    /// @see {@link Author#fetchNextMsg} for a method that retrieves the immediately next message that the user can read
    /// @see {@link Author#fetchNextMsgs} for a method that retrieves all pending messages and collects
    ///      them into an Array.
    #[wasm_bindgen(js_name = "syncState")]
    pub async fn sync_state(self) -> Result<usize> {
        self.author.borrow_mut().sync_state().await.into_js_result()
    }

    /// Fetch all the pending messages that the user can read and collect them into an Array
    ///
    /// This is the main method to traverse the a channel forward at once.  It
    /// makes sure that the messages in the Array are topologically ordered (ie
    /// parent messages before child messages), ensuring a consistent state regardless
    /// of the order of publication.
    ///
    /// @returns {UserResponse[]}
    /// @throws Throws error if an error has happened during message retrieval.
    /// @see {@link Author#fetchNextMsg} for a method that retrieves the immediately next message that the user can read
    /// @see {@link Author#syncState} for a method that traverses all pending messages to update the state
    ///      without accumulating them.
    #[wasm_bindgen(js_name = "fetchNextMsgs")]
    pub async fn fetch_next_msgs(self) -> Result<Array> {
        self.author
            .borrow_mut()
            .messages()
            .map_ok(get_message_content)
            .map_ok(JsValue::from)
            .try_collect()
            .await
            .into_js_result()
    }

    /// Fetch the immediately next message that the user can read
    ///
    /// This is the main method to traverse the a channel forward message by message, as it
    /// makes sure that no message is returned unless its parent message in the branches tree has already
    /// been returned, ensuring a consistent state regardless of the order of publication.
    ///
    /// Keep in mind that internally this method might have to fetch multiple messages until the correct
    /// message to be returned is found.
    ///
    /// @throws Throws error if an error has happened during message retrieval.
    /// @see {@link Author#fetchNextMsgs} for a method that retrieves all pending messages and collects
    ///      them into an Array.
    /// @see {@link Author#syncState} for a method that traverses all pending messages to update the state
    ///      without accumulating them.
    #[wasm_bindgen(js_name = "fetchNextMsg")]
    pub async fn fetch_next_msg(self) -> Result<Option<UserResponse>> {
        Ok(self
            .author
            .borrow_mut()
            .messages()
            .try_next()
            .await
            .into_js_result()?
            .map(get_message_content))
    }

    pub async fn fetch_prev_msg(self, link: Address) -> Result<UserResponse> {
        self.author
            .borrow_mut()
            .fetch_prev_msg(link.as_inner())
            .await
            .map(get_message_content)
            .into_js_result()
    }

    #[wasm_bindgen(catch)]
    pub async fn fetch_prev_msgs(self, link: Address, num_msgs: usize) -> Result<Array> {
        self.author
            .borrow_mut()
            .fetch_prev_msgs(link.as_inner(), num_msgs)
            .await
            .map(|msgs| {
                let responses = get_message_contents(msgs);
                responses.into_iter().map(JsValue::from).collect()
            })
            .into_js_result()
    }

    /// Generate the next batch of message {@link Address} to poll
    ///
    /// Given the set of users registered as participants of the channel and their current registered
    /// sequencing position, this method generates a set of new {@link Address} to poll for new messages
    /// (one for each user, represented by its identifier). However, beware that it is not recommended to
    /// use this method as a means to implement message traversal, as there's no guarantee that the addresses
    /// returned are the immediately next addresses to be processed. use {@link Author#fetchNextMsg} instead.
    ///
    /// Keep in mind that in multi-branch channels, the link returned corresponds to the next sequence message.
    ///
    /// @see Author#fetchNextMsg
    /// @see Author#fetchNextMsgs
    /// @returns {NextMsgAddress[]}
    #[wasm_bindgen(js_name = "genNextMsgAddresses")]
    pub fn gen_next_msg_addresses(&self) -> Array {
        self.author
            .borrow()
            .gen_next_msg_addresses()
            .into_iter()
            .map(|(id, cursor)| JsValue::from(NextMsgAddress::new(identifier_to_string(&id), cursor.link.into())))
            .collect()
    }

    #[wasm_bindgen(catch)]
    pub fn fetch_state(&self) -> Result<Array> {
        self.author
            .borrow_mut()
            .fetch_state()
            .map(|state_list| {
                state_list
                    .into_iter()
                    .map(|(id, cursor)| JsValue::from(UserState::new(id, cursor.into())))
                    .collect()
            })
            .into_js_result()
    }

    #[wasm_bindgen(catch)]
    pub fn reset_state(self) -> Result<()> {
        self.author.borrow_mut().reset_state().into_js_result()
    }

    pub fn store_new_subscriber(&self, pk_str: String) -> Result<()> {
        public_key_from_string(&pk_str)
            .and_then(|pk| self.author.borrow_mut().store_new_subscriber(pk).into_js_result())
    }

    pub fn remove_subscriber(&self, pk_str: String) -> Result<()> {
        public_key_from_string(&pk_str).and_then(|pk| self.author.borrow_mut().remove_subscriber(pk).into_js_result())
    }

    pub fn remove_psk(&self, pskid_str: String) -> Result<()> {
        pskid_from_hex_str(&pskid_str)
            .and_then(|pskid| self.author.borrow_mut().remove_psk(pskid))
            .into_js_result()
    }
}
