import { AsyncPipe } from '@angular/common';
import { Component, inject } from '@angular/core';
import { Store } from '@ngrx/store';
import { selectLoadingState } from 'app/state/loading/loading.reduce';

@Component({
  selector: 'app-loading',
  imports: [AsyncPipe],
  templateUrl: './loading.html',
  styleUrl: './loading.css',
})
export class Loading {
  store = inject(Store)
  
  loading$ = this.store.select(selectLoadingState)
}
