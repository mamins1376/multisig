// vim: set noet:

// XXX: Pipewire's Rust bindings are somewhat unsafe at the moment.
// It also uses closures for filling buffers, which is stupid.
// Why bother then?

#include <stdio.h>
#include <stdint.h>
#include <errno.h>
#include <math.h>
#include <signal.h>

#include <spa/param/audio/format-utils.h>

#include <pipewire/pipewire.h>

#define M_PI_M2 ( M_PI + M_PI )

#define DEFAULT_RATE            44100
#define DEFAULT_CHANNELS        2
#define DEFAULT_VOLUME          0.7

extern const float *multisig_process(
		void *inner,
		size_t n_channels,
		size_t n_samples,
		uint32_t rate)
	__attribute__((nonnull(1)));

struct data {
	struct pw_main_loop *loop;
	struct pw_stream *stream;
	void *inner;

	double accumulator;
};

static void on_process(void *userdata)
{
	struct data *const data = userdata;
	struct pw_buffer *const b = pw_stream_dequeue_buffer(data->stream);
	if (b == NULL) {
		pw_log_warn("out of buffers: %m");
		return;
	}

	struct spa_buffer *const buf = b->buffer;
	float *dest = buf->datas[0].data;
	if (dest == NULL)
		return;

	const size_t stride = sizeof(float) * DEFAULT_CHANNELS;
	const size_t n_frames = buf->datas[0].maxsize / stride;

	const float *const slice =
		multisig_process(data->inner, DEFAULT_CHANNELS, n_frames, DEFAULT_RATE);

	if (slice == NULL) {
		pw_main_loop_quit(data->loop);
		return;
	}

	// interlace
	for (size_t f = 0; f < n_frames; f++)
		for (size_t ch = 0; ch < DEFAULT_CHANNELS; ch++)
			*(dest++) = slice[n_frames * ch + f];

	buf->datas[0].chunk->offset = 0;
	buf->datas[0].chunk->stride = stride;
	buf->datas[0].chunk->size = n_frames * stride;

	pw_stream_queue_buffer(data->stream, b);
}

static const struct pw_stream_events stream_events = {
	PW_VERSION_STREAM_EVENTS,
	.process = on_process,
};

void multisig_pw_main(void *inner)
{
	int argc = 1;
	char *name = "multisig";
	char **argv = &name;
	pw_init(&argc, &argv);

	struct data data = {
		.loop = pw_main_loop_new(NULL),
		.stream = NULL,
		.inner = inner,
		.accumulator = 0.,
	};

	data.stream = pw_stream_new_simple(
			pw_main_loop_get_loop(data.loop),
			name,
			pw_properties_new(
				PW_KEY_MEDIA_TYPE, "Audio",
				PW_KEY_MEDIA_CATEGORY, "Playback",
				PW_KEY_MEDIA_ROLE, "Music",
				NULL),
			&stream_events,
			&data);

	uint8_t pod[1024];
	struct spa_pod_builder b = SPA_POD_BUILDER_INIT(pod, sizeof(pod));
	const struct spa_pod *param = spa_format_audio_raw_build(
		&b, SPA_PARAM_EnumFormat,
		&SPA_AUDIO_INFO_RAW_INIT(
			.format = SPA_AUDIO_FORMAT_F32,
			.channels = DEFAULT_CHANNELS,
			.rate = DEFAULT_RATE
			)
	);

	pw_stream_connect(data.stream,
			PW_DIRECTION_OUTPUT,
			PW_ID_ANY,
			PW_STREAM_FLAG_AUTOCONNECT |
			PW_STREAM_FLAG_MAP_BUFFERS |
			PW_STREAM_FLAG_RT_PROCESS,
			&param, 1);

	pw_main_loop_run(data.loop);

	pw_stream_destroy(data.stream);
	pw_main_loop_destroy(data.loop);
	pw_deinit();
}
