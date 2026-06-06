use jni::objects::{JClass, ReleaseMode};
use jni::sys::{jbyte, jbyteArray, jint, jintArray, jlong, jlongArray, jshortArray};
use jni::JNIEnv;
use paper_native_chunk_encode_core::{
    encode_light_data_into, encode_section_data_into, EncodeError, LightEncodeInput,
    SectionEncodeInput, LIGHT_UPDATE_BYTES,
};

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
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| encode_light_data_jni(
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
    )))
    .unwrap_or(ERR_ENCODE)
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
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| encode_section_data_jni_legacy(
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
    )))
    .unwrap_or(ERR_ENCODE)
}

#[no_mangle]
pub extern "system" fn Java_net_minecraft_network_protocol_game_PaperNativeChunkPacketEncode_nativeEncodeSectionDataSized(
    env: JNIEnv,
    _class: JClass,
    section_count: jint,
    non_empty_counts: jshortArray,
    state_bits: jbyteArray,
    state_palette_offsets: jintArray,
    state_palette_bytes: jbyteArray,
    state_palette_byte_count: jint,
    state_storage_offsets: jintArray,
    state_storage_longs: jlongArray,
    state_storage_long_count: jint,
    biome_bits: jbyteArray,
    biome_palette_offsets: jintArray,
    biome_palette_bytes: jbyteArray,
    biome_palette_byte_count: jint,
    biome_storage_offsets: jintArray,
    biome_storage_longs: jlongArray,
    biome_storage_long_count: jint,
    dst: jbyteArray,
    dst_len: jint,
) -> jint {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| encode_section_data_jni(
        env,
        section_count,
        non_empty_counts,
        state_bits,
        state_palette_offsets,
        state_palette_bytes,
        state_palette_byte_count,
        state_storage_offsets,
        state_storage_longs,
        state_storage_long_count,
        biome_bits,
        biome_palette_offsets,
        biome_palette_bytes,
        biome_palette_byte_count,
        biome_storage_offsets,
        biome_storage_longs,
        biome_storage_long_count,
        dst,
        Some(dst_len),
    )))
    .unwrap_or(ERR_ENCODE)
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
    let sky_update_count = match usize::try_from(sky_update_count) {
        Ok(value) => value,
        Err(_) => return ERR_ENCODE,
    };
    let block_update_count = match usize::try_from(block_update_count) {
        Ok(value) => value,
        Err(_) => return ERR_ENCODE,
    };

    let sky_y_mask_len = match get_array_len(&env, sky_y_mask_longs) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let block_y_mask_len = match get_array_len(&env, block_y_mask_longs) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let empty_sky_y_mask_len = match get_array_len(&env, empty_sky_y_mask_longs) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let empty_block_y_mask_len = match get_array_len(&env, empty_block_y_mask_longs) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let sky_update_len = match get_array_len(&env, sky_updates) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let block_update_len = match get_array_len(&env, block_updates) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let dst_len = match get_dst_len(&env, dst) {
        Ok(value) => value,
        Err(code) => return code,
    };

    let sky_update_bytes = match expected_update_bytes(sky_update_count) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let block_update_bytes = match expected_update_bytes(block_update_count) {
        Ok(value) => value,
        Err(code) => return code,
    };
    if sky_update_len < sky_update_bytes || block_update_len < block_update_bytes {
        return ERR_ENCODE;
    }

    let sky_y_mask_longs = match env.get_long_array_elements(sky_y_mask_longs, ReleaseMode::NoCopyBack) {
        Ok(value) => value,
        Err(_) => return ERR_JNI,
    };
    let block_y_mask_longs = match env.get_long_array_elements(block_y_mask_longs, ReleaseMode::NoCopyBack) {
        Ok(value) => value,
        Err(_) => return ERR_JNI,
    };
    let empty_sky_y_mask_longs = match env.get_long_array_elements(empty_sky_y_mask_longs, ReleaseMode::NoCopyBack) {
        Ok(value) => value,
        Err(_) => return ERR_JNI,
    };
    let empty_block_y_mask_longs = match env.get_long_array_elements(empty_block_y_mask_longs, ReleaseMode::NoCopyBack) {
        Ok(value) => value,
        Err(_) => return ERR_JNI,
    };
    let sky_updates = match env.get_byte_array_elements(sky_updates, ReleaseMode::NoCopyBack) {
        Ok(value) => value,
        Err(_) => return ERR_JNI,
    };
    let block_updates = match env.get_byte_array_elements(block_updates, ReleaseMode::NoCopyBack) {
        Ok(value) => value,
        Err(_) => return ERR_JNI,
    };
    let mut dst = match env.get_byte_array_elements(dst, ReleaseMode::CopyBack) {
        Ok(value) => value,
        Err(_) => return ERR_JNI,
    };

    let sky_y_mask_longs = unsafe { slice_i64(sky_y_mask_longs.as_ptr(), sky_y_mask_len) };
    let block_y_mask_longs = unsafe { slice_i64(block_y_mask_longs.as_ptr(), block_y_mask_len) };
    let empty_sky_y_mask_longs = unsafe { slice_i64(empty_sky_y_mask_longs.as_ptr(), empty_sky_y_mask_len) };
    let empty_block_y_mask_longs = unsafe { slice_i64(empty_block_y_mask_longs.as_ptr(), empty_block_y_mask_len) };
    let sky_updates = unsafe { slice_u8(sky_updates.as_ptr(), sky_update_bytes) };
    let block_updates = unsafe { slice_u8(block_updates.as_ptr(), block_update_bytes) };
    let dst_slice = unsafe { slice_u8_mut(dst.as_ptr(), dst_len) };

    let input = LightEncodeInput {
        sky_y_mask_longs,
        block_y_mask_longs,
        empty_sky_y_mask_longs,
        empty_block_y_mask_longs,
        sky_updates,
        sky_update_count,
        block_updates,
        block_update_count,
    };

    match encode_light_data_into(&input, dst_slice) {
        Ok(written) => match jint::try_from(written) {
            Ok(written) => written,
            Err(_) => {
                dst.discard();
                ERR_ENCODE
            }
        },
        Err(_) => {
            dst.discard();
            ERR_ENCODE
        }
    }
}

fn encode_section_data_jni_legacy(
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
    let section_count = match get_array_len(&env, non_empty_counts) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let state_palette_byte_count = match get_array_len(&env, state_palette_bytes) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let state_storage_long_count = match get_array_len(&env, state_storage_longs) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let biome_palette_byte_count = match get_array_len(&env, biome_palette_bytes) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let biome_storage_long_count = match get_array_len(&env, biome_storage_longs) {
        Ok(value) => value,
        Err(code) => return code,
    };

    encode_section_data_jni(
        env,
        match len_to_jint(section_count) {
            Ok(value) => value,
            Err(code) => return code,
        },
        non_empty_counts,
        state_bits,
        state_palette_offsets,
        state_palette_bytes,
        match len_to_jint(state_palette_byte_count) {
            Ok(value) => value,
            Err(code) => return code,
        },
        state_storage_offsets,
        state_storage_longs,
        match len_to_jint(state_storage_long_count) {
            Ok(value) => value,
            Err(code) => return code,
        },
        biome_bits,
        biome_palette_offsets,
        biome_palette_bytes,
        match len_to_jint(biome_palette_byte_count) {
            Ok(value) => value,
            Err(code) => return code,
        },
        biome_storage_offsets,
        biome_storage_longs,
        match len_to_jint(biome_storage_long_count) {
            Ok(value) => value,
            Err(code) => return code,
        },
        dst,
        None,
    )
}

fn encode_section_data_jni(
    env: JNIEnv,
    section_count: jint,
    non_empty_counts: jshortArray,
    state_bits: jbyteArray,
    state_palette_offsets: jintArray,
    state_palette_bytes: jbyteArray,
    state_palette_byte_count: jint,
    state_storage_offsets: jintArray,
    state_storage_longs: jlongArray,
    state_storage_long_count: jint,
    biome_bits: jbyteArray,
    biome_palette_offsets: jintArray,
    biome_palette_bytes: jbyteArray,
    biome_palette_byte_count: jint,
    biome_storage_offsets: jintArray,
    biome_storage_longs: jlongArray,
    biome_storage_long_count: jint,
    dst: jbyteArray,
    dst_len_limit: Option<jint>,
) -> jint {
    let section_count = match usize::try_from(section_count) {
        Ok(value) => value,
        Err(_) => return ERR_ENCODE,
    };
    let state_palette_byte_count = match usize::try_from(state_palette_byte_count) {
        Ok(value) => value,
        Err(_) => return ERR_ENCODE,
    };
    let state_storage_long_count = match usize::try_from(state_storage_long_count) {
        Ok(value) => value,
        Err(_) => return ERR_ENCODE,
    };
    let biome_palette_byte_count = match usize::try_from(biome_palette_byte_count) {
        Ok(value) => value,
        Err(_) => return ERR_ENCODE,
    };
    let biome_storage_long_count = match usize::try_from(biome_storage_long_count) {
        Ok(value) => value,
        Err(_) => return ERR_ENCODE,
    };

    let non_empty_counts_len = match get_array_len(&env, non_empty_counts) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let state_bits_len = match get_array_len(&env, state_bits) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let state_palette_offsets_len = match get_array_len(&env, state_palette_offsets) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let state_palette_bytes_len = match get_array_len(&env, state_palette_bytes) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let state_storage_offsets_len = match get_array_len(&env, state_storage_offsets) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let state_storage_longs_len = match get_array_len(&env, state_storage_longs) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let biome_bits_len = match get_array_len(&env, biome_bits) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let biome_palette_offsets_len = match get_array_len(&env, biome_palette_offsets) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let biome_palette_bytes_len = match get_array_len(&env, biome_palette_bytes) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let biome_storage_offsets_len = match get_array_len(&env, biome_storage_offsets) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let biome_storage_longs_len = match get_array_len(&env, biome_storage_longs) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let dst_array_len = match get_dst_len(&env, dst) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let dst_len = match dst_len_limit {
        Some(value) => match usize::try_from(value) {
            Ok(value) if value <= dst_array_len => value,
            _ => return ERR_ENCODE,
        },
        None => dst_array_len,
    };
    let offset_count = match section_count.checked_add(1) {
        Some(value) => value,
        None => return ERR_ENCODE,
    };
    if non_empty_counts_len < section_count
        || state_bits_len < section_count
        || state_palette_offsets_len < offset_count
        || state_palette_bytes_len < state_palette_byte_count
        || state_storage_offsets_len < offset_count
        || state_storage_longs_len < state_storage_long_count
        || biome_bits_len < section_count
        || biome_palette_offsets_len < offset_count
        || biome_palette_bytes_len < biome_palette_byte_count
        || biome_storage_offsets_len < offset_count
        || biome_storage_longs_len < biome_storage_long_count
    {
        return ERR_ENCODE;
    }

    let non_empty_counts = match env.get_short_array_elements(non_empty_counts, ReleaseMode::NoCopyBack) {
        Ok(value) => value,
        Err(_) => return ERR_JNI,
    };
    let state_bits = match env.get_byte_array_elements(state_bits, ReleaseMode::NoCopyBack) {
        Ok(value) => value,
        Err(_) => return ERR_JNI,
    };
    let state_palette_offsets = match env.get_int_array_elements(state_palette_offsets, ReleaseMode::NoCopyBack) {
        Ok(value) => value,
        Err(_) => return ERR_JNI,
    };
    let state_palette_bytes = match env.get_byte_array_elements(state_palette_bytes, ReleaseMode::NoCopyBack) {
        Ok(value) => value,
        Err(_) => return ERR_JNI,
    };
    let state_storage_offsets = match env.get_int_array_elements(state_storage_offsets, ReleaseMode::NoCopyBack) {
        Ok(value) => value,
        Err(_) => return ERR_JNI,
    };
    let state_storage_longs = match env.get_long_array_elements(state_storage_longs, ReleaseMode::NoCopyBack) {
        Ok(value) => value,
        Err(_) => return ERR_JNI,
    };
    let biome_bits = match env.get_byte_array_elements(biome_bits, ReleaseMode::NoCopyBack) {
        Ok(value) => value,
        Err(_) => return ERR_JNI,
    };
    let biome_palette_offsets = match env.get_int_array_elements(biome_palette_offsets, ReleaseMode::NoCopyBack) {
        Ok(value) => value,
        Err(_) => return ERR_JNI,
    };
    let biome_palette_bytes = match env.get_byte_array_elements(biome_palette_bytes, ReleaseMode::NoCopyBack) {
        Ok(value) => value,
        Err(_) => return ERR_JNI,
    };
    let biome_storage_offsets = match env.get_int_array_elements(biome_storage_offsets, ReleaseMode::NoCopyBack) {
        Ok(value) => value,
        Err(_) => return ERR_JNI,
    };
    let biome_storage_longs = match env.get_long_array_elements(biome_storage_longs, ReleaseMode::NoCopyBack) {
        Ok(value) => value,
        Err(_) => return ERR_JNI,
    };
    let mut dst = match env.get_byte_array_elements(dst, ReleaseMode::CopyBack) {
        Ok(value) => value,
        Err(_) => return ERR_JNI,
    };

    let non_empty_counts = unsafe { slice_i16(non_empty_counts.as_ptr(), section_count) };
    let state_bits = unsafe { slice_u8(state_bits.as_ptr(), section_count) };
    let state_palette_offsets = unsafe { slice_i32(state_palette_offsets.as_ptr(), offset_count) };
    let state_palette_bytes = unsafe { slice_u8(state_palette_bytes.as_ptr(), state_palette_byte_count) };
    let state_storage_offsets = unsafe { slice_i32(state_storage_offsets.as_ptr(), offset_count) };
    let state_storage_longs = unsafe { slice_i64(state_storage_longs.as_ptr(), state_storage_long_count) };
    let biome_bits = unsafe { slice_u8(biome_bits.as_ptr(), section_count) };
    let biome_palette_offsets = unsafe { slice_i32(biome_palette_offsets.as_ptr(), offset_count) };
    let biome_palette_bytes = unsafe { slice_u8(biome_palette_bytes.as_ptr(), biome_palette_byte_count) };
    let biome_storage_offsets = unsafe { slice_i32(biome_storage_offsets.as_ptr(), offset_count) };
    let biome_storage_longs = unsafe { slice_i64(biome_storage_longs.as_ptr(), biome_storage_long_count) };
    let dst_slice = unsafe { slice_u8_mut(dst.as_ptr(), dst_len) };

    let input = SectionEncodeInput {
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
    };

    match encode_section_data_into(&input, dst_slice) {
        Ok(written) => match jint::try_from(written) {
            Ok(written) => written,
            Err(_) => {
                dst.discard();
                ERR_ENCODE
            }
        },
        Err(EncodeError::DestinationTooSmall) => {
            dst.discard();
            ERR_DST_TOO_SMALL
        }
        Err(_) => {
            dst.discard();
            ERR_ENCODE
        }
    }
}

fn get_dst_len(env: &JNIEnv, dst: jbyteArray) -> Result<usize, jint> {
    env.get_array_length(dst).map(|len| len as usize).map_err(|_| ERR_JNI)
}

fn get_array_len<T>(env: &JNIEnv, array: T) -> Result<usize, jint>
where
    T: Into<jni::sys::jarray>,
{
    env.get_array_length(array.into()).map(|len| len as usize).map_err(|_| ERR_JNI)
}

fn expected_update_bytes(count: usize) -> Result<usize, jint> {
    count.checked_mul(LIGHT_UPDATE_BYTES).ok_or(ERR_ENCODE)
}

fn len_to_jint(len: usize) -> Result<jint, jint> {
    jint::try_from(len).map_err(|_| ERR_ENCODE)
}

unsafe fn slice_i64<'a>(ptr: *mut jlong, len: usize) -> &'a [i64] {
    std::slice::from_raw_parts(ptr.cast::<i64>(), len)
}

unsafe fn slice_i32<'a>(ptr: *mut jint, len: usize) -> &'a [i32] {
    std::slice::from_raw_parts(ptr.cast::<i32>(), len)
}

unsafe fn slice_i16<'a>(ptr: *mut jni::sys::jshort, len: usize) -> &'a [i16] {
    std::slice::from_raw_parts(ptr.cast::<i16>(), len)
}

unsafe fn slice_u8<'a>(ptr: *mut jbyte, len: usize) -> &'a [u8] {
    std::slice::from_raw_parts(ptr.cast::<u8>(), len)
}

unsafe fn slice_u8_mut<'a>(ptr: *mut jbyte, len: usize) -> &'a mut [u8] {
    std::slice::from_raw_parts_mut(ptr.cast::<u8>(), len)
}
