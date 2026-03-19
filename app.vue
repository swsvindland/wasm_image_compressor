<script setup lang="ts">
const { convertImage, inputFileEndings, downloadImage } = useImage();

const file = ref<File>();
const compressionFactor = ref<number>(1);
const maxSize = ref<number>(2048);
const maxFileSize = ref<number>(500);
const outputType = ref("image/webp" as keyof typeof inputFileEndings);
const isLoading = ref(false);

const trimFileExtension = (filename: string) => {
  if (!filename) return "untitled_file_compressed";

  return filename.indexOf(".") === -1
    ? filename
    : filename.split(".").slice(0, -1).join(".");
};
const startConversion = async () => {
  if (file.value) {
    isLoading.value = true;
    const reader = new FileReader();

    reader.onloadend = async (e) => {
      const res = e.target?.result;

      if (!res || !file.value) {
        isLoading.value = false;
        return;
      }

      try {
        const params = {
          outputType: outputType.value,
          compressionFactor: compressionFactor.value,
          maxSize: maxSize.value,
          maxFileSize: maxFileSize.value,
          fileOrURL: new Uint8Array(res as ArrayBuffer),
          inputType: file.value.type as keyof typeof inputFileEndings,
        };

        const response = await convertImage(params);

        if (response.success && response.data) {
          const filename = trimFileExtension(file.value.name);
          const filetype = inputFileEndings[outputType.value];

          downloadImage(response.data, `${filename}.${filetype}`);
        } else if (response.error) {
          console.error("Conversion error:", response.error);
        }
      } catch (error) {
        console.error("Unexpected error during conversion:", error);
      } finally {
        isLoading.value = false;
      }
    };

    reader.readAsArrayBuffer(file.value);
  }
};
</script>

<template>
  <Html lang="en" />
  <Title>AP | Compressor</Title>
  <Head>
    <Link rel="icon" href="/favicon.ico" />
  </Head>
  <main
    class="flex flex-col min-h-dvh items-center justify-center bg-slate-800"
  >
    <div class="w-full max-w-2xl p-5 space-y-5">
      <InputNumber
        v-model="compressionFactor"
        name="compressorFactor"
        label="Compression factor"
        placeholder="Compression factor"
      />
      <InputNumber
        v-model="maxSize"
        name="maxSize"
        label="Max size (width or height)"
        placeholder="Max size"
      />
      <InputNumber
        v-model="maxFileSize"
        name="maxFileSize"
        label="Max file size (kB)"
        placeholder="Max file size"
      />
      <InputFile v-model:file="file" />
      <InputSelect
        v-model="outputType"
        class="flex-auto"
        name="outputType"
        label="Select a File Type"
        placeholder="Select a File Type"
      >
        <option
          v-for="(imageType, ending) in inputFileEndings"
          :key="ending"
          :value="ending"
        >
          {{ imageType }}
        </option>
      </InputSelect>
      <div class="flex justify-end">
        <button
          type="button"
          class="flex items-center justify-center text-white bg-green-700 hover:bg-green-800 focus:ring-4 focus:ring-primary-300 font-medium rounded-lg text-sm px-4 py-2 mt-5 focus:outline-none disabled:bg-gray-500 disabled:cursor-not-allowed"
          :disabled="isLoading"
          @click="startConversion"
        >
          <svg
            v-if="isLoading"
            class="animate-spin -ml-1 mr-3 h-5 w-5 text-white"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
          >
            <circle
              class="opacity-25"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              stroke-width="4"
            />
            <path
              class="opacity-75"
              fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            />
          </svg>
          Convert
        </button>
      </div>
    </div>
  </main>
</template>
