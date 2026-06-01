use jni::objects::JClass;
use jni::sys::{jbyteArray, jint, jintArray, jlongArray, jshortArray};
use jni::JNIEnv;
use paper_native_chunk_encode_core::{encode_light_data, encode_section_data, LightEncodeInput, SectionEncodeInput};

const ERR_JNI: jint = -1;
const ERR_DST_TOO_SMALL: jint = -2;
const ERR_ENCODE: jint = -3;

#[no_mangle]
pub extern "system" fn Java_net_minecraft_network_protocol_game_PaperNativeChunkPacketEncode_nativeEncodeLightData(
    env: JNIEnv,
    _class: JClass,
    sky_y_mask_longs: jlongArray,
    block_y_mask_longs: jlongArray,
    empty_sky_y_mask_longs: jlongArray,
    empty_block_y_mask_longs: jlongArray,
    sky_updates: jbyteArray,
    sky_update_count: jint,
    block_updates: jbyteArray,
    block_update_count: jint,
    dst: jbyteArray,
) -> jint {
    encode_light_data_jni(
        env,
        sky_y_mask_longs,
        block_y_mask_longs,
        empty_sky_y_mask_longs,
        empty_block_y_mask_longs,
        sky_updates,
        sky_update_count,
        block_updates,
        block_update_count,
        dst,
    )
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_network_protocol_game_PaperNativeChunkPacketEncode_nativeEncodeSectionData(
    env: JNIEnv,
    _class: JClass,
    non_empty_counts: jshortArray,
    state_bits: jbyteArray,
    state_palette_offsets: jintArray,
    state_palette_bytes: jbyteArray,
    state_storage_offsets: jintArray,
    state_storage_longs: jlongArray,
    biome_bits: jbyteArray,
    biome_palette_offsets: jintArray,
    biome_palette_bytes: jbyteArray,
    biome_storage_offsets: jintArray,
    biome_storage_longs: jlongArray,
    dst: jbyteArray,
) -> jint {
    encode_section_data_jni(
        env,
        non_empty_counts,
        state_bits,
        state_palette_offsets,
        state_palette_bytes,
        state_storage_offsets,
        state_storage_longs,
        biome_bits,
        biome_palette_offsets,
        biome_palette_bytes,
        biome_storage_offsets,
        biome_storage_longs,
        dst,
    )
}

fn encode_light_data_jni(
    env: JNIEnv,
    sky_y_mask_longs: jlongArray,
    block_y_mask_longs: jlongArray,
    empty_sky_y_mask_longs: jlongArray,
    empty_block_y_mask_longs: jlongArray,
    sky_updates: jbyteArray,
    sky_update_count: jint,
    block_updates: jbyteArray,
    block_update_count: jint,
    dst: jbyteArray,
) -> jint {
    let sky_y_mask_longs = match get_i64_array(&env, sky_y_mask_longs) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let block_y_mask_longs = match get_i64_array(&env, block_y_mask_longs) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let empty_sky_y_mask_longs = match get_i64_array(&env, empty_sky_y_mask_longs) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let empty_block_y_mask_longs = match get_i64_array(&env, empty_block_y_mask_longs) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let sky_updates = match get_u8_array(&env, sky_updates) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let block_updates = match get_u8_array(&env, block_updates) {
        Ok(value) => value,
        Err(code) => return code,
    };

    let input = LightEncodeInput {
        sky_y_mask_longs: &sky_y_mask_longs,
        block_y_mask_longs: &block_y_mask_longs,
        empty_sky_y_mask_longs: &empty_sky_y_mask_longs,
        empty_block_y_mask_longs: &empty_block_y_mask_longs,
        sky_updates: &sky_updates,
        sky_update_count: match usize::try_from(sky_update_count) {
            Ok(value) => value,
            Err(_) => return ERR_ENCODE,
        },
        block_updates: &block_updates,
        block_update_count: match usize::try_from(block_update_count) {
            Ok(value) => value,
            Err(_) => return ERR_ENCODE,
        },
    };

    let mut encoded = Vec::new();
    match encode_light_data(&input, &mut encoded) {
        Ok(_) => write_dst(&env, dst, &encoded),
        Err(_) => ERR_ENCODE,
    }
}

fn encode_section_data_jni(
    env: JNIEnv,
    non_empty_counts: jshortArray,
    state_bits: jbyteArray,
    state_palette_offsets: jintArray,
    state_palette_bytes: jbyteArray,
    state_storage_offsets: jintArray,
    state_storage_longs: jlongArray,
    biome_bits: jbyteArray,
    biome_palette_offsets: jintArray,
    biome_palette_bytes: jbyteArray,
    biome_storage_offsets: jintArray,
    biome_storage_longs: jlongArray,
    dst: jbyteArray,
) -> jint {
    let non_empty_counts = match get_i16_array(&env, non_empty_counts) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let state_bits = match get_u8_array(&env, state_bits) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let state_palette_offsets = match get_i32_array(&env, state_palette_offsets) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let state_palette_bytes = match get_u8_array(&env, state_palette_bytes) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let state_storage_offsets = match get_i32_array(&env, state_storage_offsets) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let state_storage_longs = match get_i64_array(&env, state_storage_longs) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let biome_bits = match get_u8_array(&env, biome_bits) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let biome_palette_offsets = match get_i32_array(&env, biome_palette_offsets) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let biome_palette_bytes = match get_u8_array(&env, biome_palette_bytes) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let biome_storage_offsets = match get_i32_array(&env, biome_storage_offsets) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let biome_storage_longs = match get_i64_array(&env, biome_storage_longs) {
        Ok(value) => value,
        Err(code) => return code,
    };

    let input = SectionEncodeInput {
        non_empty_counts: &non_empty_counts,
        state_bits: &state_bits,
        state_palette_offsets: &state_palette_offsets,
        state_palette_bytes: &state_palette_bytes,
        state_storage_offsets: &state_storage_offsets,
        state_storage_longs: &state_storage_longs,
        biome_bits: &biome_bits,
        biome_palette_offsets: &biome_palette_offsets,
        biome_palette_bytes: &biome_palette_bytes,
        biome_storage_offsets: &biome_storage_offsets,
        biome_storage_longs: &biome_storage_longs,
    };

    let mut encoded = Vec::new();
    match encode_section_data(&input, &mut encoded) {
        Ok(_) => write_dst(&env, dst, &encoded),
        Err(_) => ERR_ENCODE,
    }
}

fn get_u8_array(env: &JNIEnv, array: jbyteArray) -> Result<Vec<u8>, jint> {
    env.convert_byte_array(array).map_err(|_| ERR_JNI)
}

fn get_i16_array(env: &JNIEnv, array: jshortArray) -> Result<Vec<i16>, jint> {
    let len = env.get_array_length(array).map_err(|_| ERR_JNI)? as usize;
    let mut out = vec![0i16; len];
    env.get_short_array_region(array, 0, &mut out).map_err(|_| ERR_JNI)?;
    Ok(out)
}

fn get_i32_array(env: &JNIEnv, array: jintArray) -> Result<Vec<i32>, jint> {
    let len = env.get_array_length(array).map_err(|_| ERR_JNI)? as usize;
    let mut out = vec![0i32; len];
    env.get_int_array_region(array, 0, &mut out).map_err(|_| ERR_JNI)?;
    Ok(out)
}

fn get_i64_array(env: &JNIEnv, array: jlongArray) -> Result<Vec<i64>, jint> {
    let len = env.get_array_length(array).map_err(|_| ERR_JNI)? as usize;
    let mut out = vec![0i64; len];
    env.get_long_array_region(array, 0, &mut out).map_err(|_| ERR_JNI)?;
    Ok(out)
}

fn write_dst(env: &JNIEnv, dst: jbyteArray, encoded: &[u8]) -> jint {
    let dst_len = match env.get_array_length(dst) {
        Ok(len) => len as usize,
        Err(_) => return ERR_JNI,
    };
    if encoded.len() > dst_len {
        return ERR_DST_TOO_SMALL;
    }

    let signed: Vec<i8> = encoded.iter().map(|byte| *byte as i8).collect();
    match env.set_byte_array_region(dst, 0, &signed) {
        Ok(()) => encoded.len() as jint,
        Err(_) => ERR_JNI,
    }
}
