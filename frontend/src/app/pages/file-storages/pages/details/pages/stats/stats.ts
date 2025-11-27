import { AsyncPipe, DatePipe } from '@angular/common';
import { Component, inject, signal } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { UnitConvertPipe } from '@core/pipes/unit-convert-pipe';
import { FileStoragesService } from '@core/services/file-storages';
import { Store } from '@ngrx/store';
import { setLoading } from 'app/state/loading/loading.actions';
import Snackbar from 'awesome-snackbar';
import { catchError, tap } from 'rxjs';

@Component({
  selector: 'app-stats',
  imports: [AsyncPipe, DatePipe, UnitConvertPipe],
  templateUrl: './stats.html',
  styleUrl: './stats.css',
})
export class Stats {
  route = inject(ActivatedRoute);
  router = inject(Router);
  fileStoragesService = inject(FileStoragesService);
  store = inject(Store)
  Math = Math;
  fileStorageId = this.route.snapshot.parent!.paramMap.get('id')!;

  fileStorage$ = this.fileStoragesService.getFileStorageById(this.fileStorageId).pipe(
    tap(() => this.store.dispatch(setLoading({ state: false }))),
    catchError((error: any) => {
      if (error.status === 404) {
        this.router.navigate(['/404']);
      }
      new Snackbar(error.error?.message || `Failed to load file storage details`, {
        iconSrc: '/error.png',
        position: 'bottom-right',
      });
      this.router.navigate(['/file-storages']);
      return [];
    })
  )

  copied = signal(false)

  constructor() {
    this.store.dispatch(setLoading({ state: true }));
  }

  onCopyApiKey(apiKey: string) {
    navigator.clipboard.writeText(apiKey);
    this.copied.set(true);
    new Snackbar('API key copied to clipboard', {
      iconSrc: '/success.png',
      position: 'bottom-right',
    });
    setTimeout(() => {
      this.copied.set(false);
    }, 1000);
  }
}
