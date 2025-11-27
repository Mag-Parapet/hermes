import { ComponentFixture, TestBed } from '@angular/core/testing';

import { UploadFiles } from './upload-files';

describe('UploadFiles', () => {
  let component: UploadFiles;
  let fixture: ComponentFixture<UploadFiles>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [UploadFiles]
    })
    .compileComponents();

    fixture = TestBed.createComponent(UploadFiles);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
