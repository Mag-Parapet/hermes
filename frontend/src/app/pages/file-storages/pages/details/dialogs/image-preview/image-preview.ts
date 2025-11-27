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
    if (!this.blurhashCanvas || !this.file.variants.blurhash) return; // Note: using this.canvasRef from the component structure

    const canvas = this.blurhashCanvas.nativeElement;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const blurWidth = 32;
    const blurHeight = 32;

    // 1. Set the internal canvas dimensions to the small decoding size
    canvas.width = blurWidth;
    canvas.height = blurHeight;

    // 2. Decode pixels directly to that small size
    const pixels = decode(this.file.variants.blurhash, blurWidth, blurHeight);
    
    // 3. Put pixels directly onto the context
    const imageData = ctx.createImageData(blurWidth, blurHeight);
    imageData.data.set(pixels);
    ctx.putImageData(imageData, 0, 0);
    
    // The browser automatically scales this small 32x32 image to fit the 
    // full size of the <canvas> element defined by your CSS.
}
}
