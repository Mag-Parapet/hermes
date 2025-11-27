import { DatePipe, JsonPipe } from '@angular/common';
import { Component, inject } from '@angular/core';
import { UnitConvertPipe } from '@core/pipes/unit-convert-pipe';
import { DialogRef } from '@ngneat/dialog';

@Component({
  selector: 'app-file-details',
  imports: [UnitConvertPipe, JsonPipe, DatePipe],
  templateUrl: './file-details.html',
  styleUrl: './file-details.css',
})
export class FileDetails {
  ref = inject(DialogRef);
  file = this.ref.data.file;

  Object = Object;
}
