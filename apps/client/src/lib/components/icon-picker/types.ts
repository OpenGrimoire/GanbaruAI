export interface IconPickerAsset {
  relativePath: string;
  originalName?: string | null;
  contentType?: string;
  byteSize?: number;
  sha256?: string;
}

export interface IconPickerUploadAdapter {
  pickImageFile: () => Promise<IconPickerAsset | null>;
  saveImageDataUrl: (dataUrl: string, originalName?: string | null) => Promise<IconPickerAsset>;
  assetUrl: (asset: IconPickerAsset) => Promise<string>;
  deleteAssetsIfUnreferenced?: (relativePaths: string[]) => Promise<void>;
  downloadImageUrl?: (url: string) => Promise<IconPickerAsset>;
  selectAsset: (asset: IconPickerAsset) => void | Promise<void>;
  selectPickedAssetImmediately?: boolean;
  selectExternalUrl?: (url: string) => void | Promise<void>;
}

export interface IconPickerTriggerContext {
  open: boolean;
  toggle: () => void;
}
