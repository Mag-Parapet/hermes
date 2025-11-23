import { createAction, props } from '@ngrx/store';

export const setLoading = createAction(
  '[Loading Bar] Set State',
  props<{ state: boolean }>()
);
