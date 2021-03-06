#![allow(unused_variables)]
//The macro is much cleaner if we allow for unused variables
use super::constants::{CHUNK_SIZE, ENTITY_ID_BLOCK_SIZE};
use super::minecraft_protocol::{MinecraftProtocolReader, MinecraftProtocolWriter};
use super::minecraft_types::ChunkSection;
use super::translation::TranslationInfo;
use std::any::type_name;
use std::io::{Cursor, Read, Write};

// Format: (state (99 is outgoing), name, id, [ list of (field name, field type) ]
#[rustfmt::skip::macros(packet_boilerplate)]
packet_boilerplate!(
    (
        0,
        Handshake,
        0,
        [
            (protocol_version, VarInt),
            (server_address, String),
            (server_port, UShort),
            (next_state, VarInt)
        ]
    ),
    (1, StatusRequest, 0, []),
    (1, Ping, 1, [(payload, Long)]),
    (2, LoginStart, 0, [(username, String)]),
    (3, KeepAlive, 0x21, [(id, Long)]),
    (
        3,
        PlayerPosition,
        0x10,
        [
            (x, Double, XEntity),
            (feet_y, Double),
            (z, Double),
            (on_ground, Boolean)
        ]
    ),
    (
        _, //Temporary for border crossing, once it has its own packet change this back to 3
        PlayerPositionAndLook,
        0x11,
        [
            (x, Double, XEntity),
            (feet_y, Double),
            (z, Double),
            (yaw, Float),
            (pitch, Float),
            (on_ground, Boolean)
        ]
    ),
    (
        3,
        PlayerLook,
        0x12,
        [
            (yaw, Float),
            (pitch, Float),
            (on_ground, Boolean)
        ]
    ),
    (6, ReportState, 0x1, []),
    (_, BorderCrossLogin, 0xA0, [
            (x, Double, XEntity),
            (feet_y, Double),
            (z, Double),
            (yaw, Float),
            (pitch, Float),
            (on_ground, Boolean),
            (username, String),
            (entity_id, Int, EntityId)
    ]),
    (99, Pong, 1, [(payload, Long)]),
    (99, StatusResponse, 0, [(json_response, String)]),
    (99, LoginSuccess, 2, [(uuid, String), (username, String)]),
    (
        99,
        JoinGame,
        0x25,
        [
            (entity_id, Int, EntityId),
            (gamemode, UByte),
            (dimension, Int),
            (difficulty, UByte),
            (max_players, UByte),
            (level_type, String),
            (reduced_debug_info, Boolean)
        ]
    ),
    (
        99,
        ClientboundPlayerPositionAndLook,
        0x32,
        [
            (x, Double),
            (y, Double),
            (z, Double),
            (yaw, Float),
            (pitch, Float),
            (flags, Byte),
            (teleport_id, VarInt)
        ]
    ),
    (
        _,
        ChunkData,
        0x22,
        [
            (chunk_x, Int, XChunk),
            (chunk_z, Int),
            (full_chunk, Boolean), //always true
            (primary_bit_mask, VarInt),
            (size, VarInt),
            (data, ChunkSection), //actually a chunk array, but can pretend its 1 for now
            (biomes, Array(Int, 256)),
            (number_of_block_entities, VarInt)
        ]
    ),
    (
        _,
        PlayerInfo,
        0x30,
        [
            (action, VarInt),
            (number_of_players, VarInt),
            (uuid, u128),
            (name, String),
            (number_of_properties, VarInt),
            (gamemode, VarInt),
            (ping, VarInt),
            (has_display_name, Boolean)
        ]
    ),
    (
        _,
        SpawnPlayer,
        0x05,
        [
            (entity_id, VarInt, EntityId),
            (uuid, u128),
            (x, Double, XEntity),
            (y, Double),
            (z, Double),
            (yaw, UByte), // represents angle * (360/256). Might want to eventually make this its own type
            (pitch, UByte),
            (entity_metadata_terminator, UByte)  // always 0xff until we implement entity metadata
        ]
    ),
    (
        _,
        EntityHeadLook,
        0x39,
        [
            (entity_id, VarInt, EntityId),
            (angle, UByte)
        ]
    ),
    (
        _,
        DestroyEntities,
        0x35,
        [
            (entity_ids, LengthPrefixedArray(VarInt), Array(EntityId))
        ]
    ),
    (
        _,
        EntityLookAndMove,
        0x29,
        [
            (entity_id, VarInt, EntityId),
            (delta_x, Short),
            (delta_y, Short),
            (delta_z, Short),
            (yaw, UByte),
            (pitch, UByte),
            (on_ground, Boolean)
        ]
    )
);
