import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ContentTable } from './content-table';

describe('ContentTable', () => {
  let component: ContentTable;
  let fixture: ComponentFixture<ContentTable>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ContentTable]
    })
    .compileComponents();

    fixture = TestBed.createComponent(ContentTable);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
