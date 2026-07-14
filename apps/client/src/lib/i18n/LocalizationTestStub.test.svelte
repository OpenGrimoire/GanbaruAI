<script lang="ts">
  import { getLocalization } from "./translator.svelte";

  const localization = getLocalization();
</script>

<span data-localization-test-stub data-locale={localization.locale}>
  {localization.t("common.save")}
</span>
