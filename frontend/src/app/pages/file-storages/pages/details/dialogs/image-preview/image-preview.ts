import { AfterViewInit, Component, computed, effect, ElementRef, inject, signal, ViewChild } from '@angular/core';
import { DialogRef } from '@ngneat/dialog';
import { decode } from 'blurhash';

@Component({
  selector: 'app-image-preview',
  imports: [],
  templateUrl: './image-preview.html',
  styleUrl: './image-preview.css',
})
export class ImagePreview implements AfterViewInit {
  ref = inject(DialogRef);
  file = this.ref.data.file;
  @ViewChild('blurhashCanvas') blurhashCanvas!: ElementRef<HTMLCanvasElement>;
  @ViewChild('blurhashContainer') blurhashContainer!: ElementRef<HTMLDivElement>;
  @ViewChild('sampleImage') sampleImage!: ElementRef<HTMLImageElement>;

  selectedVariant = signal('original');

  Object = Object;

  variants = signal([...Object.keys(this.file.variants)]);

  sizes = signal({
    width: this.file.width,
    height: this.file.height,
  });

  selectedVariantImage = computed(() => {
    if (this.selectedVariant() === 'original') {
      return this.file.url;
    }
    return this.file.variants[this.selectedVariant()];
  });

  constructor() {
    effect(() => {
      if (this.selectedVariant() === 'blurhash') {
        setTimeout(() => this.renderBlurhash(), 0);
      }
    });
  }

  ngAfterViewInit(): void {
    this.sizes.set({
      width: this.sampleImage.nativeElement.clientWidth,
      height: this.sampleImage.nativeElement.clientHeight,
    });
    let container = this.blurhashContainer.nativeElement;
    container.style.width = this.sizes().width + 'px';
    container.style.height = this.sizes().height + 2 + 'px';
  }

  renderBlurhash() {
    if (!this.blurhashCanvas || !this.sampleImage) return;

    const canvas = this.blurhashCanvas.nativeElement;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    const blurWidth = 128;
    const blurHeight = 128;
    const pixels = decode(this.file.variants.blurhash, blurWidth, blurHeight);

    const off = document.createElement('canvas');
    off.width = blurWidth;
    off.height = blurHeight;
    const offCtx = off.getContext('2d')!;
    const imgData = offCtx.createImageData(blurWidth, blurHeight);
    imgData.data.set(pixels);
    offCtx.putImageData(imgData, 0, 0);

    ctx.drawImage(off, 0, 0, this.sizes().width, this.sizes().height);
  }
}
